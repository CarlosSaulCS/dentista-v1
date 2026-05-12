import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ImageUp, Plus } from "lucide-react";
import { useSearchParams } from "react-router-dom";
import { toast } from "sonner";
import { PageHeader } from "@/components/data/PageHeader";
import { StatusBadge } from "@/components/data/StatusBadge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Textarea } from "@/components/ui/textarea";
import { listPatients } from "@/features/patients/services/patient-service";
import { formatCurrency, formatDateTime } from "@/lib/api";
import { officeApi } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

const emptyPaymentDraft = { patientId: "", concept: "", amount: "", method: "efectivo", notes: "" };

export function PaymentsPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken) ?? "";
  const queryClient = useQueryClient();
  const [searchParams, setSearchParams] = useSearchParams();
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState(() => ({ ...emptyPaymentDraft, patientId: searchParams.get("patientId") ?? "" }));
  const [proofFile, setProofFile] = useState<File | null>(null);
  const newPaymentRequested = searchParams.get("new") === "1";
  const dialogOpen = open || newPaymentRequested;
  const payments = useQuery({ queryKey: ["payments", sessionToken], queryFn: () => officeApi.listPayments(sessionToken), enabled: Boolean(sessionToken) });
  const patients = useQuery({ queryKey: ["patients", sessionToken, "payments"], queryFn: () => listPatients(sessionToken, "", 200), enabled: Boolean(sessionToken) });
  const mutation = useMutation({
    mutationFn: async () => {
      const payment = await officeApi.registerPayment(sessionToken, { patientId: draft.patientId, concept: draft.concept, amountCents: Math.round(Number(draft.amount || 0) * 100), method: draft.method, notes: draft.notes });
      let proofSaved = false;
      let proofError: string | null = null;
      if (proofFile) {
        try {
          await officeApi.savePatientFile(sessionToken, {
            patientId: draft.patientId,
            categoryName: "Comprobantes de pago",
            originalName: proofFile.name,
            mimeType: proofFile.type,
            description: `Comprobante del pago ${payment.folio}`,
            relatedEntityType: "payments",
            relatedEntityId: payment.id,
            bytes: Array.from(new Uint8Array(await proofFile.arrayBuffer())),
          });
          proofSaved = true;
        } catch (error) {
          proofError = error instanceof Error ? error.message : String(error);
        }
      }
      return { proofSaved, proofError };
    },
    onSuccess: async ({ proofSaved, proofError }) => {
      if (proofError) {
        toast.warning(`Pago registrado, pero el comprobante no se guardó: ${proofError}`);
      } else {
        toast.success(proofFile && proofSaved ? "Pago y comprobante registrados" : "Pago registrado");
      }
      setOpen(false);
      setDraft({ ...emptyPaymentDraft, patientId: searchParams.get("patientId") ?? "" });
      setProofFile(null);
      if (newPaymentRequested) {
        const nextParams = new URLSearchParams(searchParams);
        nextParams.delete("new");
        setSearchParams(nextParams, { replace: true });
      }
      await queryClient.invalidateQueries({ queryKey: ["payments"] });
      await queryClient.invalidateQueries({ queryKey: ["patient-files"] });
      await queryClient.invalidateQueries({ queryKey: ["cash-register"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard-summary"] });
      await queryClient.invalidateQueries({ queryKey: ["reports"] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const handleOpenChange = (value: boolean) => {
    setOpen(value);
    if (!value && newPaymentRequested) {
      const nextParams = new URLSearchParams(searchParams);
      nextParams.delete("new");
      setSearchParams(nextParams, { replace: true });
    }
  };

  const handlePatientChange = (patientId: string) => {
    setDraft({ ...draft, patientId });
    const nextParams = new URLSearchParams(searchParams);
    nextParams.set("patientId", patientId);
    setSearchParams(nextParams, { replace: true });
  };

  const submitPayment = () => {
    const amount = Number(draft.amount);
    if (!draft.patientId) {
      toast.error("Selecciona un paciente");
      return;
    }
    if (!draft.concept.trim()) {
      toast.error("Escribe el concepto del pago");
      return;
    }
    if (!Number.isFinite(amount) || amount <= 0) {
      toast.error("El monto debe ser mayor a cero");
      return;
    }
    mutation.mutate();
  };

  return <div className="space-y-6">
    <PageHeader title="Pagos y abonos" description="Registro local de pagos con folio y método." actions={
      <Dialog open={dialogOpen} onOpenChange={handleOpenChange}>
        <DialogTrigger asChild><Button><Plus className="h-4 w-4" />Registrar pago</Button></DialogTrigger>
        <DialogContent className="max-h-[92vh] max-w-3xl overflow-y-auto p-0">
          <div className="border-b px-6 py-5">
            <DialogHeader>
              <DialogTitle>Registrar pago</DialogTitle>
              <DialogDescription>Guarda el pago localmente y adjunta comprobante cuando aplique.</DialogDescription>
            </DialogHeader>
          </div>
          <form className="grid gap-5 px-6 py-5" onSubmit={(event) => { event.preventDefault(); submitPayment(); }}>
            <Pick label="Paciente" value={draft.patientId} onChange={handlePatientChange} items={(patients.data ?? []).map((p) => ({ value: p.id, label: p.fullName }))} />
            <div className="grid gap-4 sm:grid-cols-[1fr_180px]">
              <Field label="Concepto" value={draft.concept} onChange={(concept) => setDraft({ ...draft, concept })} />
              <Field label="Monto MXN" value={draft.amount} onChange={(amount) => setDraft({ ...draft, amount })} type="number" />
            </div>
            <Pick label="Método" value={draft.method} onChange={(method) => setDraft({ ...draft, method })} items={paymentMethods} />
            <div className="grid gap-2"><Label>Notas opcionales</Label><Textarea value={draft.notes} onChange={(event) => setDraft({ ...draft, notes: event.target.value })} /></div>
            <div className="grid gap-2">
              <Label>Comprobante o captura opcional</Label>
              <label className="flex min-w-0 cursor-pointer items-center justify-between gap-3 rounded-lg border bg-card px-4 py-3 text-sm transition hover:border-primary/50 hover:bg-primary/5">
                <span className="min-w-0 truncate text-muted-foreground">{proofFile?.name ?? "Imagen/PDF de transferencia, depósito o comprobante"}</span>
                <span className="inline-flex shrink-0 items-center gap-2 font-medium text-primary"><ImageUp className="h-4 w-4" />Cargar</span>
                <Input className="hidden" type="file" accept="image/*,application/pdf" onChange={(event) => setProofFile(event.target.files?.[0] ?? null)} />
              </label>
            </div>
            <Button type="submit" disabled={mutation.isPending}>Guardar pago</Button>
          </form>
        </DialogContent>
      </Dialog>
    } />
    <Card><CardContent className="overflow-x-auto p-0"><Table>
      <TableHeader><TableRow><TableHead>Folio</TableHead><TableHead>Paciente</TableHead><TableHead>Concepto</TableHead><TableHead>Monto</TableHead><TableHead>Método</TableHead><TableHead>Comprobante</TableHead><TableHead>Fecha</TableHead><TableHead>Estado</TableHead></TableRow></TableHeader>
      <TableBody>{(payments.data ?? []).length === 0 ? (
        <TableRow><TableCell colSpan={8} className="h-24 text-center text-muted-foreground">No hay pagos registrados.</TableCell></TableRow>
      ) : (payments.data ?? []).map((payment) => <TableRow key={payment.id}>
        <TableCell className="font-medium">{payment.folio}</TableCell><TableCell>{payment.patientName}</TableCell><TableCell>{payment.concept}</TableCell>
        <TableCell>{formatCurrency(payment.amountCents)}</TableCell><TableCell>{payment.method}</TableCell><TableCell>{payment.proofFilesCount > 0 ? `${payment.proofFilesCount} archivo(s)` : "Sin archivo"}</TableCell><TableCell>{formatDateTime(payment.paidAt)}</TableCell><TableCell><StatusBadge status={payment.status} /></TableCell>
      </TableRow>)}</TableBody>
    </Table></CardContent></Card>
  </div>;
}

const paymentMethods = [
  { value: "efectivo", label: "Efectivo" },
  { value: "transferencia", label: "Transferencia" },
  { value: "tarjeta", label: "Tarjeta" },
  { value: "mixto", label: "Mixto" },
  { value: "otro", label: "Otro" },
];

function Field({ label, value, onChange, type = "text" }: { label: string; value: string; onChange: (value: string) => void; type?: string }) {
  return <div className="grid gap-2"><Label>{label}</Label><Input type={type} value={value} onChange={(event) => onChange(event.target.value)} /></div>;
}

function Pick({ label, value, onChange, items }: { label: string; value: string; onChange: (value: string) => void; items: { value: string; label: string }[] }) {
  return <div className="grid gap-2"><Label>{label}</Label><Select value={value} onValueChange={onChange}><SelectTrigger><SelectValue placeholder={label} /></SelectTrigger><SelectContent>{items.map((item) => <SelectItem key={item.value} value={item.value}>{item.label}</SelectItem>)}</SelectContent></Select></div>;
}
