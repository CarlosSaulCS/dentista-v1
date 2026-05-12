import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus } from "lucide-react";
import { toast } from "sonner";
import { PageHeader } from "@/components/data/PageHeader";
import { StatusBadge } from "@/components/data/StatusBadge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from "@/components/ui/sheet";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Textarea } from "@/components/ui/textarea";
import { listPatients } from "@/features/patients/services/patient-service";
import { officeApi } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

export function AlertsPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken) ?? "";
  const queryClient = useQueryClient();
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState({ patientId: "", alertType: "seguimiento", priority: "media", title: "", message: "", dueAt: "" });
  const alerts = useQuery({ queryKey: ["alerts", sessionToken], queryFn: () => officeApi.listAlerts(sessionToken), enabled: Boolean(sessionToken) });
  const patients = useQuery({ queryKey: ["patients", sessionToken, "alerts"], queryFn: () => listPatients(sessionToken, "", 200), enabled: Boolean(sessionToken) });
  const createMutation = useMutation({ mutationFn: (input: typeof draft) => officeApi.createAlert(sessionToken, input), onSuccess: async () => { toast.success("Alerta creada"); setOpen(false); await queryClient.invalidateQueries({ queryKey: ["alerts"] }); await queryClient.invalidateQueries({ queryKey: ["dashboard-summary"] }); }, onError: (error) => toast.error(error instanceof Error ? error.message : String(error)) });
  const resolveMutation = useMutation({ mutationFn: (id: string) => officeApi.resolveAlert(sessionToken, id), onSuccess: async () => { await queryClient.invalidateQueries({ queryKey: ["alerts"] }); await queryClient.invalidateQueries({ queryKey: ["dashboard-summary"] }); } });
  return <div className="space-y-6"><PageHeader title="Alertas" description="Seguimientos, saldos, citas, inventario y pendientes." actions={
    <Sheet open={open} onOpenChange={setOpen}><SheetTrigger asChild><Button><Plus className="h-4 w-4" />Nueva</Button></SheetTrigger><SheetContent><SheetHeader><SheetTitle>Nueva alerta</SheetTitle></SheetHeader><div className="mt-6 grid gap-4">
      <Pick label="Paciente" value={draft.patientId} onChange={(patientId) => setDraft({ ...draft, patientId })} items={[{ value: "none", label: "Sin paciente" }, ...(patients.data ?? []).map((p) => ({ value: p.id, label: p.fullName }))]} />
      <Field label="Tipo" value={draft.alertType} onChange={(alertType) => setDraft({ ...draft, alertType })} />
      <Pick label="Prioridad" value={draft.priority} onChange={(priority) => setDraft({ ...draft, priority })} items={["critica", "alta", "media", "baja"].map((item) => ({ value: item, label: item }))} />
      <Field label="Título" value={draft.title} onChange={(title) => setDraft({ ...draft, title })} />
      <Field label="Vence" value={draft.dueAt} onChange={(dueAt) => setDraft({ ...draft, dueAt })} type="date" />
      <div className="grid gap-2"><Label>Mensaje</Label><Textarea value={draft.message} onChange={(event) => setDraft({ ...draft, message: event.target.value })} /></div>
      <Button onClick={() => createMutation.mutate({ ...draft, patientId: draft.patientId === "none" ? "" : draft.patientId })}>Guardar</Button>
    </div></SheetContent></Sheet>} />
    <Card><CardContent className="p-0"><Table><TableHeader><TableRow><TableHead>Prioridad</TableHead><TableHead>Alerta</TableHead><TableHead>Paciente</TableHead><TableHead>Vence</TableHead><TableHead>Estado</TableHead><TableHead></TableHead></TableRow></TableHeader><TableBody>{(alerts.data ?? []).map((alert) => <TableRow key={alert.id}><TableCell>{alert.priority}</TableCell><TableCell><div className="font-medium">{alert.title}</div><div className="text-xs text-muted-foreground">{alert.message}</div></TableCell><TableCell>{alert.patientName ?? "General"}</TableCell><TableCell>{alert.dueAt ?? "Sin fecha"}</TableCell><TableCell><StatusBadge status={alert.status} /></TableCell><TableCell><Button variant="outline" size="sm" onClick={() => resolveMutation.mutate(alert.id)}>Resolver</Button></TableCell></TableRow>)}</TableBody></Table></CardContent></Card>
  </div>;
}

function Field({ label, value, onChange, type = "text" }: { label: string; value: string; onChange: (value: string) => void; type?: string }) {
  return <div className="grid gap-2"><Label>{label}</Label><Input type={type} value={value} onChange={(event) => onChange(event.target.value)} /></div>;
}
function Pick({ label, value, onChange, items }: { label: string; value: string; onChange: (value: string) => void; items: { value: string; label: string }[] }) {
  return <div className="grid gap-2"><Label>{label}</Label><Select value={value} onValueChange={(value) => onChange(value === "none" ? "" : value)}><SelectTrigger><SelectValue placeholder={label} /></SelectTrigger><SelectContent>{items.map((item) => <SelectItem key={item.value} value={item.value}>{item.label}</SelectItem>)}</SelectContent></Select></div>;
}
