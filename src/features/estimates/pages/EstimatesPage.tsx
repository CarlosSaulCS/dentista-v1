import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { save } from "@tauri-apps/plugin-dialog";
import { FileDown, Plus } from "lucide-react";
import { useSearchParams } from "react-router-dom";
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
  const [searchParams, setSearchParams] = useSearchParams();
  const patientIdParam = searchParams.get("patientId") ?? "";
  const newEstimateRequested = searchParams.get("new") === "1";
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState(() => ({ ...emptyEstimateDraft, patientId: patientIdParam }));
  const [exportingEstimateId, setExportingEstimateId] = useState<string | null>(null);
  const sheetOpen = open || newEstimateRequested;
  const estimates = useQuery({ queryKey: ["estimates", sessionToken], queryFn: () => officeApi.listEstimates(sessionToken), enabled: Boolean(sessionToken) });
  const patients = useQuery({ queryKey: ["patients", sessionToken, "estimates"], queryFn: () => listPatients(sessionToken, "", 200), enabled: Boolean(sessionToken) });

  const clearNewEstimateRequest = () => {
    if (!newEstimateRequested) return;
    const nextParams = new URLSearchParams(searchParams);
    nextParams.delete("new");
    setSearchParams(nextParams, { replace: true });
  };

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
      clearNewEstimateRequest();
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
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
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

  const exportEstimate = async (estimate: EstimateSummary) => {
    try {
      setExportingEstimateId(estimate.id);
      const items = await officeApi.listEstimateItems(sessionToken, estimate.id);
      if (items.length === 0) {
        toast.error("El presupuesto no tiene conceptos para exportar");
        return;
      }

      const fileName = `Presupuesto_${safeFileName(estimate.folio)}_${safeFileName(estimate.patientName)}.csv`;
      const targetPath = await save({
        defaultPath: fileName,
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });
      if (!targetPath) return;

      const csv = buildEstimateCsv(estimate, items);
      const bytes = Array.from(new TextEncoder().encode(csv));
      const result = await officeApi.saveReportFile(sessionToken, {
        reportType: "estimate",
        format: "csv",
        fileName,
        targetPath,
        filtersJson: JSON.stringify({ estimateId: estimate.id, folio: estimate.folio }),
        bytes,
      });
      toast.success(`CSV guardado en ${result.path}`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setExportingEstimateId(null);
    }
  };

  return <div className="space-y-6">
    <PageHeader title="Presupuestos" description="Cotizaciones con folio, estado, vigencia y exportación local." actions={
      <Sheet open={sheetOpen} onOpenChange={(value) => {
        setOpen(value);
        if (!value) clearNewEstimateRequest();
      }}>
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
          <Button variant="outline" size="sm" onClick={() => void exportEstimate(estimate)} disabled={exportingEstimateId === estimate.id}><FileDown className="h-4 w-4" />CSV</Button>
          <Button variant="outline" size="sm" onClick={() => statusMutation.mutate({ id: estimate.id, status: "approved" })}>Aprobar</Button>
        </TableCell>
      </TableRow>)}</TableBody>
    </Table></CardContent></Card>
  </div>;
}

function buildEstimateCsv(estimate: EstimateSummary, items: Awaited<ReturnType<typeof officeApi.listEstimateItems>>) {
  const delimiter = ";";
  const rows = [
    ["Dentista v1 Professional - Presupuesto"],
    [],
    ["Informacion del presupuesto"],
    ["Campo", "Valor"],
    ["Folio", estimate.folio],
    ["Paciente", estimate.patientName],
    ["Fecha de emision", formatCsvDate(estimate.createdAt)],
    ["Vigencia", estimate.validUntil ? formatCsvDate(estimate.validUntil) : "Sin vigencia"],
    ["Estado", formatStatus(estimate.status)],
    [],
    ["Conceptos"],
    ["No.", "Concepto", "Cantidad", "Precio unitario MXN", "Descuento MXN", "Total MXN"],
    ...items.map((item, index) => [
      String(index + 1),
      item.description,
      String(item.quantity),
      centsToCsvAmount(item.unitPriceCents),
      centsToCsvAmount(item.discountCents),
      centsToCsvAmount(item.totalCents),
    ]),
    [],
    ["Resumen"],
    ["Subtotal MXN", centsToCsvAmount(estimate.subtotalCents)],
    ["Descuento MXN", centsToCsvAmount(estimate.discountCents)],
    ["Total MXN", centsToCsvAmount(estimate.totalCents)],
    [],
    ["Observaciones"],
    [estimate.observations || "Sin observaciones"],
    [],
    ["Terminos"],
    [estimate.terms || "Sin terminos registrados"],
    [],
    ["Generado", new Intl.DateTimeFormat("es-MX", { dateStyle: "medium", timeStyle: "short" }).format(new Date())],
  ];
  return `\uFEFFsep=;\r\n${rows.map((row) => row.map(csvCell).join(delimiter)).join("\r\n")}`;
}

function centsToCsvAmount(cents: number) {
  return (cents / 100).toFixed(2);
}

function csvCell(value: string | number | null | undefined) {
  const text = String(value ?? "").replace(/\r?\n/g, " ").trim();
  return `"${text.replaceAll("\"", "\"\"")}"`;
}

function formatCsvDate(value: string) {
  if (!value) return "";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return new Intl.DateTimeFormat("es-MX", { dateStyle: "medium" }).format(date);
}

function formatStatus(status: string) {
  return status.replaceAll("_", " ");
}

function safeFileName(value: string) {
  return value
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^a-zA-Z0-9_-]+/g, "_")
    .replace(/^_+|_+$/g, "")
    .slice(0, 48) || "presupuesto";
}

function Field({ label, value, onChange, type = "text" }: { label: string; value: string; onChange: (value: string) => void; type?: string }) {
  return <div className="grid gap-2"><Label>{label}</Label><Input type={type} value={value} onChange={(event) => onChange(event.target.value)} /></div>;
}

function Pick({ label, value, onChange, items }: { label: string; value: string; onChange: (value: string) => void; items: { value: string; label: string }[] }) {
  return <div className="grid gap-2"><Label>{label}</Label><Select value={value} onValueChange={onChange}><SelectTrigger><SelectValue placeholder={label} /></SelectTrigger><SelectContent>{items.map((item) => <SelectItem key={item.value} value={item.value}>{item.label}</SelectItem>)}</SelectContent></Select></div>;
}
