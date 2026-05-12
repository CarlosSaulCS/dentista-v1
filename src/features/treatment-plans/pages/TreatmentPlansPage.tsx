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
import { formatCurrency } from "@/lib/api";
import { officeApi } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

export function TreatmentPlansPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken) ?? "";
  const queryClient = useQueryClient();
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState({ patientId: "", treatmentId: "", diagnosis: "", tooth: "", phase: "Fase 1", priority: "media", price: "", notes: "" });
  const plans = useQuery({ queryKey: ["treatment-plans", sessionToken], queryFn: () => officeApi.listTreatmentPlans(sessionToken), enabled: Boolean(sessionToken) });
  const patients = useQuery({ queryKey: ["patients", sessionToken, "plans"], queryFn: () => listPatients(sessionToken, "", 200), enabled: Boolean(sessionToken) });
  const treatments = useQuery({ queryKey: ["treatments", sessionToken], queryFn: () => officeApi.listTreatments(sessionToken), enabled: Boolean(sessionToken) });
  const mutation = useMutation({
    mutationFn: () => {
      const treatment = treatments.data?.find((item) => item.id === draft.treatmentId);
      const price = draft.price ? Math.round(Number(draft.price) * 100) : treatment?.basePriceCents ?? 0;
      return officeApi.createTreatmentPlan(sessionToken, {
        patientId: draft.patientId,
        diagnosis: draft.diagnosis,
        notes: draft.notes,
        items: [{ treatmentCatalogId: draft.treatmentId, toothNumber: draft.tooth, diagnosis: draft.diagnosis, phase: draft.phase, priority: draft.priority, quantity: 1, unitPriceCents: price, discountCents: 0, notes: draft.notes }],
      });
    },
    onSuccess: async () => {
      toast.success("Plan creado");
      setOpen(false);
      await queryClient.invalidateQueries({ queryKey: ["treatment-plans"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard-summary"] });
      await queryClient.invalidateQueries({ queryKey: ["reports"] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  return (
    <div className="space-y-6">
      <PageHeader title="Planes de tratamiento" description="Planes por paciente con total, avance financiero y estado." actions={
        <Sheet open={open} onOpenChange={setOpen}>
          <SheetTrigger asChild><Button><Plus className="h-4 w-4" />Nuevo plan</Button></SheetTrigger>
          <SheetContent className="overflow-y-auto">
            <SheetHeader><SheetTitle>Nuevo plan</SheetTitle></SheetHeader>
            <div className="mt-6 grid gap-4">
              <Pick label="Paciente" value={draft.patientId} onChange={(patientId) => setDraft({ ...draft, patientId })} items={(patients.data ?? []).map((p) => ({ value: p.id, label: p.fullName }))} />
              <Pick label="Tratamiento" value={draft.treatmentId} onChange={(treatmentId) => setDraft({ ...draft, treatmentId })} items={(treatments.data ?? []).map((t) => ({ value: t.id, label: `${t.name} · ${formatCurrency(t.basePriceCents)}` }))} />
              <Field label="Pieza dental" value={draft.tooth} onChange={(tooth) => setDraft({ ...draft, tooth })} />
              <Field label="Precio MXN" value={draft.price} onChange={(price) => setDraft({ ...draft, price })} type="number" />
              <Field label="Prioridad" value={draft.priority} onChange={(priority) => setDraft({ ...draft, priority })} />
              <div className="grid gap-2"><Label>Diagnóstico / observaciones</Label><Textarea value={draft.diagnosis} onChange={(event) => setDraft({ ...draft, diagnosis: event.target.value })} /></div>
              <Button onClick={() => mutation.mutate()} disabled={mutation.isPending}>Crear plan</Button>
            </div>
          </SheetContent>
        </Sheet>
      } />
      <Card><CardContent className="p-0"><Table>
        <TableHeader><TableRow><TableHead>Paciente</TableHead><TableHead>Diagnóstico</TableHead><TableHead>Total</TableHead><TableHead>Pagado</TableHead><TableHead>Saldo</TableHead><TableHead>Estado</TableHead></TableRow></TableHeader>
        <TableBody>{(plans.data ?? []).map((plan) => <TableRow key={plan.id}>
          <TableCell className="font-medium">{plan.patientName}</TableCell>
          <TableCell>{plan.diagnosis || "Sin diagnóstico"}</TableCell>
          <TableCell>{formatCurrency(plan.totalCents)}</TableCell>
          <TableCell>{formatCurrency(plan.paidCents)}</TableCell>
          <TableCell>{formatCurrency(plan.balanceCents)}</TableCell>
          <TableCell><StatusBadge status={plan.status} /></TableCell>
        </TableRow>)}</TableBody>
      </Table></CardContent></Card>
    </div>
  );
}

function Field({ label, value, onChange, type = "text" }: { label: string; value: string; onChange: (value: string) => void; type?: string }) {
  return <div className="grid gap-2"><Label>{label}</Label><Input type={type} value={value} onChange={(event) => onChange(event.target.value)} /></div>;
}

function Pick({ label, value, onChange, items }: { label: string; value: string; onChange: (value: string) => void; items: { value: string; label: string }[] }) {
  return <div className="grid gap-2"><Label>{label}</Label><Select value={value} onValueChange={onChange}><SelectTrigger><SelectValue placeholder={label} /></SelectTrigger><SelectContent>{items.map((item) => <SelectItem key={item.value} value={item.value}>{item.label}</SelectItem>)}</SelectContent></Select></div>;
}
