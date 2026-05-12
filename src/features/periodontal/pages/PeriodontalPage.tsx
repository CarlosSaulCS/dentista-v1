import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { PageHeader } from "@/components/data/PageHeader";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Textarea } from "@/components/ui/textarea";
import { listPatients } from "@/features/patients/services/patient-service";
import { formatDateTime } from "@/lib/api";
import { officeApi } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

export function PeriodontalPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken) ?? "";
  const queryClient = useQueryClient();
  const [patientId, setPatientId] = useState("");
  const [notes, setNotes] = useState("");
  const patients = useQuery({ queryKey: ["patients", sessionToken, "periodontal"], queryFn: () => listPatients(sessionToken, "", 200), enabled: Boolean(sessionToken) });
  const records = useQuery({ queryKey: ["periodontal", sessionToken], queryFn: () => officeApi.listPeriodontalRecords(sessionToken), enabled: Boolean(sessionToken) });
  const mutation = useMutation({ mutationFn: () => officeApi.createPeriodontalRecord(sessionToken, { patientId, notes }), onSuccess: async () => { toast.success("Registro periodontal guardado"); setNotes(""); await queryClient.invalidateQueries({ queryKey: ["periodontal"] }); }, onError: (error) => toast.error(error instanceof Error ? error.message : String(error)) });
  return <div className="space-y-6"><PageHeader title="Periodontograma" description="Base local para registro periodontal con notas y seguimiento por paciente." />
    <Card><CardContent className="grid gap-4 p-4 md:grid-cols-[320px_1fr_160px] md:items-end"><div className="grid gap-2"><Label>Paciente</Label><Select value={patientId} onValueChange={setPatientId}><SelectTrigger><SelectValue placeholder="Paciente" /></SelectTrigger><SelectContent>{(patients.data ?? []).map((p) => <SelectItem key={p.id} value={p.id}>{p.fullName}</SelectItem>)}</SelectContent></Select></div><div className="grid gap-2"><Label>Notas periodontales</Label><Textarea value={notes} onChange={(event) => setNotes(event.target.value)} /></div><Button onClick={() => mutation.mutate()}>Guardar</Button></CardContent></Card>
    <Card><CardContent className="p-0"><Table><TableHeader><TableRow><TableHead>Paciente</TableHead><TableHead>Notas</TableHead><TableHead>Estado</TableHead><TableHead>Fecha</TableHead></TableRow></TableHeader><TableBody>{(records.data ?? []).map((record) => <TableRow key={record.id}><TableCell className="font-medium">{record.patientName}</TableCell><TableCell>{record.notes}</TableCell><TableCell>{record.status}</TableCell><TableCell>{formatDateTime(record.createdAt)}</TableCell></TableRow>)}</TableBody></Table></CardContent></Card>
  </div>;
}
