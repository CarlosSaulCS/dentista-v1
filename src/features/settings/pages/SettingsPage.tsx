import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { PageHeader } from "@/components/data/PageHeader";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { getBootstrapStatus } from "@/features/auth/services/auth-service";
import { officeApi } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

export function SettingsPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken) ?? "";
  const queryClient = useQueryClient();
  const bootstrap = useQuery({ queryKey: ["bootstrap-status"], queryFn: getBootstrapStatus });
  const templates = useQuery({ queryKey: ["message-templates", sessionToken], queryFn: () => officeApi.listMessageTemplates(sessionToken), enabled: Boolean(sessionToken) });
  const clinic = bootstrap.data?.clinic;
  const license = bootstrap.data?.license;
  const [draft, setDraft] = useState({ name: "", phone: "", whatsapp: "", email: "", address: "", taxData: "" });
  const mutation = useMutation({
    mutationFn: () =>
      officeApi.updateClinicSettings(sessionToken, {
        ...draft,
        name: draft.name || clinic?.name || "",
        phone: draft.phone || clinic?.phone || "",
        whatsapp: draft.whatsapp || clinic?.whatsapp || "",
        email: draft.email || clinic?.email || "",
        address: draft.address || clinic?.address || "",
      }),
    onSuccess: async () => { toast.success("Configuración guardada"); await queryClient.invalidateQueries({ queryKey: ["bootstrap-status"] }); },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });
  return <div className="space-y-6"><PageHeader title="Configuración" description="Datos locales del consultorio, folios y plantillas." />
    {license ? <Card><CardHeader><CardTitle>Licencia local</CardTitle></CardHeader><CardContent className="space-y-2 text-sm">
      <p className="font-medium">{license.isLicensed ? "Sistema activado" : license.requiresActivation ? "Prueba finalizada" : "Prueba activa"}</p>
      {!license.isLicensed && license.trialEndsAt ? <p className="text-muted-foreground">Vigencia de prueba: {formatLicenseDate(license.trialEndsAt)}. Días restantes: {license.daysRemaining}.</p> : null}
      {license.isLicensed && license.activatedAt ? <p className="text-muted-foreground">Activado el {formatLicenseDate(license.activatedAt)}.</p> : null}
    </CardContent></Card> : null}
    <Card><CardHeader><CardTitle>Consultorio</CardTitle></CardHeader><CardContent className="grid gap-4 md:grid-cols-2">
      <Field label="Nombre" value={draft.name || clinic?.name || ""} onChange={(name) => setDraft({ ...draft, name })} />
      <Field label="Teléfono" value={draft.phone || ""} onChange={(phone) => setDraft({ ...draft, phone })} />
      <Field label="WhatsApp" value={draft.whatsapp || ""} onChange={(whatsapp) => setDraft({ ...draft, whatsapp })} />
      <Field label="Correo" value={draft.email || ""} onChange={(email) => setDraft({ ...draft, email })} />
      <Field label="Dirección" value={draft.address || ""} onChange={(address) => setDraft({ ...draft, address })} />
      <Field label="Datos fiscales" value={draft.taxData || ""} onChange={(taxData) => setDraft({ ...draft, taxData })} />
      <Button className="md:col-span-2" onClick={() => mutation.mutate()}>Guardar configuración</Button>
    </CardContent></Card>
    <Card><CardHeader><CardTitle>Plantillas de WhatsApp listas para copiar</CardTitle></CardHeader><CardContent className="p-0"><Table><TableHeader><TableRow><TableHead>Nombre</TableHead><TableHead>Texto</TableHead></TableRow></TableHeader><TableBody>{(templates.data ?? []).map((template) => <TableRow key={template.id}><TableCell className="font-medium">{template.name}</TableCell><TableCell>{template.body}</TableCell></TableRow>)}</TableBody></Table></CardContent></Card>
  </div>;
}

function Field({ label, value, onChange }: { label: string; value: string; onChange: (value: string) => void }) {
  return <div className="grid gap-2"><Label>{label}</Label><Input value={value} onChange={(event) => onChange(event.target.value)} /></div>;
}

function formatLicenseDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value.replace("T", " ").slice(0, 16);
  return new Intl.DateTimeFormat("es-MX", { dateStyle: "medium", timeStyle: "short" }).format(date);
}
