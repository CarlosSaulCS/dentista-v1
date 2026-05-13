import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ExternalLink, FileText } from "lucide-react";
import { useSearchParams } from "react-router-dom";
import { toast } from "sonner";
import { EmptyState } from "@/components/data/EmptyState";
import { PageHeader } from "@/components/data/PageHeader";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { createClinicalEvolution, createClinicalRecord, listClinicalEvolutions, listClinicalRecords } from "@/features/clinical-records/services/clinical-service";
import { listPatients } from "@/features/patients/services/patient-service";
import { formatDateTime } from "@/lib/api";
import { officeApi, type PatientFileSummary } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

export function ClinicalRecordsPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const [patientId, setPatientId] = useState(() => searchParams.get("patientId") ?? "");
  const [recordDraft, setRecordDraft] = useState({ chiefComplaint: "", diagnosis: "", suggestedPlan: "" });
  const [evolutionDraft, setEvolutionDraft] = useState({ reason: "", findings: "", proceduresDone: "", indications: "" });
  const sessionToken = useAuthStore((state) => state.sessionToken);
  const queryClient = useQueryClient();

  const patients = useQuery({
    queryKey: ["patients", sessionToken, "clinical-picker"],
    queryFn: () => listPatients(sessionToken ?? "", "", 200),
    enabled: Boolean(sessionToken),
  });
  const records = useQuery({
    queryKey: ["clinical-records", sessionToken, patientId],
    queryFn: () => listClinicalRecords(sessionToken ?? "", patientId),
    enabled: Boolean(sessionToken && patientId),
  });
  const evolutions = useQuery({
    queryKey: ["clinical-evolutions", sessionToken, patientId],
    queryFn: () => listClinicalEvolutions(sessionToken ?? "", patientId),
    enabled: Boolean(sessionToken && patientId),
  });
  const patientFiles = useQuery({
    queryKey: ["patient-files", sessionToken],
    queryFn: () => officeApi.listPatientFiles(sessionToken ?? ""),
    enabled: Boolean(sessionToken && patientId),
  });

  const recordMutation = useMutation({
    mutationFn: () => createClinicalRecord(sessionToken ?? "", { patientId, ...recordDraft }),
    onSuccess: async () => {
      toast.success("Historia clínica guardada");
      setRecordDraft({ chiefComplaint: "", diagnosis: "", suggestedPlan: "" });
      await queryClient.invalidateQueries({ queryKey: ["clinical-records"] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const evolutionMutation = useMutation({
    mutationFn: () => createClinicalEvolution(sessionToken ?? "", { patientId, ...evolutionDraft }),
    onSuccess: async () => {
      toast.success("Evolución clínica registrada");
      setEvolutionDraft({ reason: "", findings: "", proceduresDone: "", indications: "" });
      await queryClient.invalidateQueries({ queryKey: ["clinical-evolutions"] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const handlePatientChange = (value: string) => {
    setPatientId(value);
    setRecordDraft({ chiefComplaint: "", diagnosis: "", suggestedPlan: "" });
    setEvolutionDraft({ reason: "", findings: "", proceduresDone: "", indications: "" });
    const nextParams = new URLSearchParams(searchParams);
    nextParams.set("patientId", value);
    setSearchParams(nextParams, { replace: true });
  };
  const visibleFiles = (patientFiles.data ?? []).filter((file) => file.patientId === patientId);
  const openFile = async (file: PatientFileSummary) => {
    try {
      await officeApi.openPatientFile(sessionToken ?? "", file.id);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  };

  return (
    <div className="space-y-6">
      <PageHeader title="Expediente clínico" description="Historia clínica y evoluciones con responsable y auditoría." />

      <Card>
        <CardContent className="grid gap-2 p-4">
          <Label>Paciente</Label>
          <Select value={patientId} onValueChange={handlePatientChange}>
            <SelectTrigger className="max-w-xl">
              <SelectValue placeholder="Selecciona paciente" />
            </SelectTrigger>
            <SelectContent>
              {(patients.data ?? []).map((patient) => (
                <SelectItem key={patient.id} value={patient.id}>
                  {patient.fullName}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </CardContent>
      </Card>

      {!patientId ? (
        <EmptyState title="Selecciona un paciente" description="El expediente se carga por paciente para mantener el contexto clínico claro." />
      ) : (
        <Tabs defaultValue="record" className="space-y-4">
          <TabsList>
            <TabsTrigger value="record">Historia clínica</TabsTrigger>
            <TabsTrigger value="evolution">Evoluciones</TabsTrigger>
            <TabsTrigger value="documents">Documentos</TabsTrigger>
          </TabsList>

          <TabsContent value="record" className="grid gap-4 xl:grid-cols-[0.9fr_1.1fr]">
            <Card>
              <CardHeader>
                <CardTitle>Nueva historia clínica</CardTitle>
              </CardHeader>
              <CardContent className="grid gap-3">
                <ClinicalTextarea label="Motivo de consulta" value={recordDraft.chiefComplaint} onChange={(chiefComplaint) => setRecordDraft((draft) => ({ ...draft, chiefComplaint }))} />
                <ClinicalTextarea label="Diagnóstico" value={recordDraft.diagnosis} onChange={(diagnosis) => setRecordDraft((draft) => ({ ...draft, diagnosis }))} />
                <ClinicalTextarea label="Plan sugerido" value={recordDraft.suggestedPlan} onChange={(suggestedPlan) => setRecordDraft((draft) => ({ ...draft, suggestedPlan }))} />
                <Button onClick={() => recordMutation.mutate()} disabled={recordMutation.isPending}>
                  Guardar historia
                </Button>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Historial</CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                {(records.data ?? []).map((record) => (
                  <div key={record.id} className="rounded-md border p-3">
                    <div className="flex items-center justify-between gap-3">
                      <div className="font-medium">{record.chiefComplaint || "Historia clínica"}</div>
                      <div className="text-xs text-muted-foreground">{formatDateTime(record.createdAt)}</div>
                    </div>
                    <p className="mt-2 text-sm text-muted-foreground">{record.diagnosis || "Sin diagnóstico registrado"}</p>
                  </div>
                ))}
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="evolution" className="grid gap-4 xl:grid-cols-[0.9fr_1.1fr]">
            <Card>
              <CardHeader>
                <CardTitle>Nueva evolución</CardTitle>
              </CardHeader>
              <CardContent className="grid gap-3">
                <ClinicalTextarea label="Motivo" value={evolutionDraft.reason} onChange={(reason) => setEvolutionDraft((draft) => ({ ...draft, reason }))} />
                <ClinicalTextarea label="Hallazgos" value={evolutionDraft.findings} onChange={(findings) => setEvolutionDraft((draft) => ({ ...draft, findings }))} />
                <ClinicalTextarea label="Procedimientos realizados" value={evolutionDraft.proceduresDone} onChange={(proceduresDone) => setEvolutionDraft((draft) => ({ ...draft, proceduresDone }))} />
                <ClinicalTextarea label="Indicaciones" value={evolutionDraft.indications} onChange={(indications) => setEvolutionDraft((draft) => ({ ...draft, indications }))} />
                <Button onClick={() => evolutionMutation.mutate()} disabled={evolutionMutation.isPending}>
                  Registrar evolución
                </Button>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Evoluciones registradas</CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                {(evolutions.data ?? []).map((evolution) => (
                  <div key={evolution.id} className="rounded-md border p-3">
                    <div className="flex items-center justify-between gap-3">
                      <div className="font-medium">{evolution.reason}</div>
                      <div className="text-xs text-muted-foreground">{formatDateTime(evolution.createdAt)}</div>
                    </div>
                    <p className="mt-2 text-sm text-muted-foreground">{evolution.proceduresDone || evolution.findings || "Sin detalle adicional"}</p>
                  </div>
                ))}
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="documents">
            <Card>
              <CardHeader>
                <CardTitle>Documentos del expediente</CardTitle>
              </CardHeader>
              <CardContent className="overflow-x-auto p-0">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Documento</TableHead>
                      <TableHead>Categoría</TableHead>
                      <TableHead>Descripción</TableHead>
                      <TableHead>Tamaño</TableHead>
                      <TableHead>Fecha</TableHead>
                      <TableHead>Acción</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {visibleFiles.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={6} className="h-24 text-center text-muted-foreground">
                          No hay documentos guardados para este paciente.
                        </TableCell>
                      </TableRow>
                    ) : (
                      visibleFiles.map((file) => (
                        <TableRow key={file.id}>
                          <TableCell className="font-medium">
                            <span className="inline-flex items-center gap-2">
                              <FileText className="h-4 w-4 text-primary" />
                              {file.originalName}
                            </span>
                          </TableCell>
                          <TableCell>{file.categoryName ?? "Sin categoría"}</TableCell>
                          <TableCell>{file.description ?? "Sin descripción"}</TableCell>
                          <TableCell>{(file.sizeBytes / 1024).toFixed(1)} KB</TableCell>
                          <TableCell>{formatDateTime(file.createdAt)}</TableCell>
                          <TableCell>
                            <Button variant="outline" size="sm" onClick={() => void openFile(file)}>
                              <ExternalLink className="h-4 w-4" />
                              Abrir
                            </Button>
                          </TableCell>
                        </TableRow>
                      ))
                    )}
                  </TableBody>
                </Table>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      )}
    </div>
  );
}

function ClinicalTextarea({ label, value, onChange }: { label: string; value: string; onChange: (value: string) => void }) {
  return (
    <div className="grid gap-2">
      <Label>{label}</Label>
      <Textarea value={value} onChange={(event) => onChange(event.target.value)} />
    </div>
  );
}
