import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { FileDown, Plus } from "lucide-react";
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
import { officeApi, type EstimateSummary } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

const emptyEstimateDraft = { patientId: "", description: "", quantity: "1", price: "", validUntil: "", observations: "" };

export function EstimatesPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken) ?? "";
  const queryClient = useQueryClient();
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState(emptyEstimateDraft);
  const estimates = useQuery({ queryKey: ["estimates", sessionToken], queryFn: () => officeApi.listEstimates(sessionToken), enabled: Boolean(sessionToken) });
  const patients = useQuery({ queryKey: ["patients", sessionToken, "estimates"], queryFn: () => listPatients(sessionToken, "", 200), enabled: Boolean(sessionToken) });
  const mutation = useMutation({
    mutationFn: () => officeApi.createEstimate(sessionToken, {
      patientId: draft.patientId,
      validUntil: draft.validUntil,
      observations: draft.observations,
      terms: "Vigencia sujeta a valoración clínica y disponibilidad de agenda.",
      items: [{ description: draft.description, quantity: Number(draft.quantity || 1), unitPriceCents: Math.round(Number(draft.price || 0) * 100), discountCents: 0 }],
    }),
    onSuccess: async () => {
      toast.success("Presupuesto generado");
      setOpen(false);
      setDraft(emptyEstimateDraft);
      await queryClient.invalidateQueries({ queryKey: ["estimates"] });
      await queryClient.invalidateQueries({ queryKey: ["reports"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard-summary"] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });
  const statusMutation = useMutation({
    mutationFn: ({ id, status }: { id: string; status: string }) => officeApi.updateEstimateStatus(sessionToken, id, status),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["estimates"] });
      await queryClient.invalidateQueries({ queryKey: ["reports"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard-summary"] });
      await queryClient.invalidateQueries({ queryKey: ["alerts"] });
    },
  });

  const submitEstimate = () => {
    const quantity = Number(draft.quantity || 1);
    const price = Number(draft.price);
    if (!draft.patientId) {
      toast.error("Selecciona un paciente");
      return;
    }
    if (!draft.description.trim()) {
      toast.error("Escribe el concepto del presupuesto");
      return;
    }
    if (!Number.isFinite(quantity) || quantity <= 0) {
      toast.error("La cantidad debe ser mayor a cero");
      return;
    }
    if (!Number.isFinite(price) || price <= 0) {
      toast.error("El precio debe ser mayor a cero");
      return;
    }
    mutation.mutate();
  };

  return <div className="space-y-6">
    <PageHeader title="Presupuestos" description="Cotizaciones con folio, estado, vigencia y exportación local." actions={
      <Sheet open={open} onOpenChange={setOpen}>
        <SheetTrigger asChild><Button><Plus className="h-4 w-4" />Nuevo</Button></SheetTrigger>
        <SheetContent>
          <SheetHeader><SheetTitle>Nuevo presupuesto</SheetTitle></SheetHeader>
          <div className="mt-6 grid gap-4">
            <Pick label="Paciente" value={draft.patientId} onChange={(patientId) => setDraft({ ...draft, patientId })} items={(patients.data ?? []).map((p) => ({ value: p.id, label: p.fullName }))} />
            <Field label="Concepto" value={draft.description} onChange={(description) => setDraft({ ...draft, description })} />
            <Field label="Cantidad" value={draft.quantity} onChange={(quantity) => setDraft({ ...draft, quantity })} type="number" />
            <Field label="Precio MXN" value={draft.price} onChange={(price) => setDraft({ ...draft, price })} type="number" />
            <Field label="Vigencia" value={draft.validUntil} onChange={(validUntil) => setDraft({ ...draft, validUntil })} type="date" />
            <div className="grid gap-2"><Label>Observaciones</Label><Textarea value={draft.observations} onChange={(event) => setDraft({ ...draft, observations: event.target.value })} /></div>
            <Button onClick={submitEstimate} disabled={mutation.isPending}>Generar</Button>
          </div>
        </SheetContent>
      </Sheet>
    } />
    <Card><CardContent className="overflow-x-auto p-0"><Table>
      <TableHeader><TableRow><TableHead>Folio</TableHead><TableHead>Paciente</TableHead><TableHead>Total</TableHead><TableHead>Vigencia</TableHead><TableHead>Estado</TableHead><TableHead>Acciones</TableHead></TableRow></TableHeader>
      <TableBody>{(estimates.data ?? []).length === 0 ? (
        <TableRow><TableCell colSpan={6} className="h-24 text-center text-muted-foreground">No hay presupuestos registrados.</TableCell></TableRow>
      ) : (estimates.data ?? []).map((estimate) => <TableRow key={estimate.id}>
        <TableCell className="font-medium">{estimate.folio}</TableCell><TableCell>{estimate.patientName}</TableCell>
        <TableCell>{formatCurrency(estimate.totalCents)}</TableCell><TableCell>{estimate.validUntil || "Sin vigencia"}</TableCell>
        <TableCell><StatusBadge status={estimate.status} /></TableCell>
        <TableCell className="flex gap-2">
          <Button variant="outline" size="sm" onClick={() => exportEstimate(estimate)}><FileDown className="h-4 w-4" />CSV</Button>
          <Button variant="outline" size="sm" onClick={() => statusMutation.mutate({ id: estimate.id, status: "approved" })}>Aprobar</Button>
        </TableCell>
      </TableRow>)}</TableBody>
    </Table></CardContent></Card>
  </div>;
}

function exportEstimate(estimate: EstimateSummary) {
  const csv = `Folio,Paciente,Total,Estado\n${estimate.folio},${estimate.patientName},${estimate.totalCents / 100},${estimate.status}\n`;
  const url = URL.createObjectURL(new Blob([csv], { type: "text/csv;charset=utf-8" }));
  const a = document.createElement("a");
  a.href = url;
  a.download = `${estimate.folio}.csv`;
  a.click();
  URL.revokeObjectURL(url);
}

function Field({ label, value, onChange, type = "text" }: { label: string; value: string; onChange: (value: string) => void; type?: string }) {
  return <div className="grid gap-2"><Label>{label}</Label><Input type={type} value={value} onChange={(event) => onChange(event.target.value)} /></div>;
}

function Pick({ label, value, onChange, items }: { label: string; value: string; onChange: (value: string) => void; items: { value: string; label: string }[] }) {
  return <div className="grid gap-2"><Label>{label}</Label><Select value={value} onValueChange={onChange}><SelectTrigger><SelectValue placeholder={label} /></SelectTrigger><SelectContent>{items.map((item) => <SelectItem key={item.value} value={item.value}>{item.label}</SelectItem>)}</SelectContent></Select></div>;
}
