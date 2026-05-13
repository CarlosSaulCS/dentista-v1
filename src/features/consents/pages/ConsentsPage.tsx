import { useState } from "react";
import { PDFDocument, StandardFonts, rgb } from "pdf-lib";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { FileText, Plus } from "lucide-react";
import { toast } from "sonner";
import { PageHeader } from "@/components/data/PageHeader";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Textarea } from "@/components/ui/textarea";
import { listPatients } from "@/features/patients/services/patient-service";
import { officeApi } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

export function ConsentsPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken) ?? "";
  const queryClient = useQueryClient();
  const [templateDraft, setTemplateDraft] = useState({ name: "", treatmentCategory: "", body: "" });
  const [patientId, setPatientId] = useState("");
  const [templateId, setTemplateId] = useState("");
  const templates = useQuery({ queryKey: ["consent-templates", sessionToken], queryFn: () => officeApi.listConsentTemplates(sessionToken), enabled: Boolean(sessionToken) });
  const patients = useQuery({ queryKey: ["patients", sessionToken, "consents"], queryFn: () => listPatients(sessionToken, "", 200), enabled: Boolean(sessionToken) });
  const createTemplate = useMutation({ mutationFn: () => officeApi.createConsentTemplate(sessionToken, templateDraft), onSuccess: async () => { toast.success("Plantilla guardada"); setTemplateDraft({ name: "", treatmentCategory: "", body: "" }); await queryClient.invalidateQueries({ queryKey: ["consent-templates"] }); }, onError: (error) => toast.error(error instanceof Error ? error.message : String(error)) });
  const generate = useMutation({
    mutationFn: async () => {
      const patient = patients.data?.find((item) => item.id === patientId);
      const template = templates.data?.find((item) => item.id === templateId);
      if (!patient || !template) throw new Error("Selecciona paciente y plantilla");
      const pdfBytes = await buildConsentPdf(template.name, template.body.replaceAll("{{paciente}}", patient.fullName));
      return officeApi.savePatientFile(sessionToken, {
        patientId,
        categoryName: "Consentimientos",
        originalName: `${template.name}-${patient.fullName}.pdf`,
        mimeType: "application/pdf",
        description: `Consentimiento informado: ${template.name}`,
        relatedEntityType: "consent_templates",
        relatedEntityId: template.id,
        bytes: Array.from(pdfBytes),
      });
    },
    onSuccess: async () => { toast.success("Consentimiento PDF guardado en documentos del expediente"); await queryClient.invalidateQueries({ queryKey: ["patient-files"] }); },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });
  return <div className="space-y-6"><PageHeader title="Consentimientos informados" description="Plantillas locales y generación de PDF en expediente." />
    <div className="grid gap-4 xl:grid-cols-2">
      <Card><CardContent className="grid gap-3 p-4"><h2 className="font-semibold">Nueva plantilla</h2><Input placeholder="Nombre" value={templateDraft.name} onChange={(e) => setTemplateDraft({ ...templateDraft, name: e.target.value })} /><Input placeholder="Categoría/tratamiento" value={templateDraft.treatmentCategory} onChange={(e) => setTemplateDraft({ ...templateDraft, treatmentCategory: e.target.value })} /><Textarea placeholder="Texto. Puedes usar {{paciente}}" value={templateDraft.body} onChange={(e) => setTemplateDraft({ ...templateDraft, body: e.target.value })} /><Button onClick={() => createTemplate.mutate()}><Plus className="h-4 w-4" />Guardar plantilla</Button></CardContent></Card>
      <Card><CardContent className="grid gap-3 p-4"><h2 className="font-semibold">Generar PDF</h2><Select value={patientId} onValueChange={setPatientId}><SelectTrigger><SelectValue placeholder="Paciente" /></SelectTrigger><SelectContent>{(patients.data ?? []).map((p) => <SelectItem key={p.id} value={p.id}>{p.fullName}</SelectItem>)}</SelectContent></Select><Select value={templateId} onValueChange={setTemplateId}><SelectTrigger><SelectValue placeholder="Plantilla" /></SelectTrigger><SelectContent>{(templates.data ?? []).map((t) => <SelectItem key={t.id} value={t.id}>{t.name}</SelectItem>)}</SelectContent></Select><Button onClick={() => generate.mutate()}><FileText className="h-4 w-4" />Generar y guardar</Button></CardContent></Card>
    </div>
    <Card><CardContent className="p-0"><Table><TableHeader><TableRow><TableHead>Plantilla</TableHead><TableHead>Categoría</TableHead><TableHead>Texto</TableHead></TableRow></TableHeader><TableBody>{(templates.data ?? []).map((template) => <TableRow key={template.id}><TableCell className="font-medium">{template.name}</TableCell><TableCell>{template.treatmentCategory}</TableCell><TableCell className="max-w-xl truncate">{template.body}</TableCell></TableRow>)}</TableBody></Table></CardContent></Card>
  </div>;
}

async function buildConsentPdf(title: string, body: string) {
  const pdf = await PDFDocument.create();
  const page = pdf.addPage([612, 792]);
  const font = await pdf.embedFont(StandardFonts.Helvetica);
  page.drawText(title, { x: 48, y: 736, size: 18, font, color: rgb(0.12, 0.16, 0.25) });
  const words = body.split(/\s+/);
  let line = ""; let y = 700;
  for (const word of words) {
    const next = `${line} ${word}`.trim();
    if (next.length > 88) { page.drawText(line, { x: 48, y, size: 11, font }); line = word; y -= 18; } else { line = next; }
  }
  if (line) page.drawText(line, { x: 48, y, size: 11, font });
  page.drawText("Firma del paciente: ________________________________", { x: 48, y: 96, size: 11, font });
  return pdf.save();
}
