import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Pencil, Plus } from "lucide-react";
import { toast } from "sonner";
import { PageHeader } from "@/components/data/PageHeader";
import { SelectableCards } from "@/components/data/SelectableCards";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Textarea } from "@/components/ui/textarea";
import { formatCurrency } from "@/lib/api";
import { officeApi, type TreatmentCatalogItem } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

const emptyDraft = {
  id: "",
  name: "",
  category: "Diagnóstico",
  description: "",
  price: "",
  duration: "",
  requiresFollowUp: "no",
  active: true,
};

const treatmentCategories = [
  "Diagnóstico",
  "Preventivo",
  "Restaurativo",
  "Endodoncia",
  "Cirugía",
  "Prótesis",
  "Ortodoncia",
  "Estética",
  "Periodoncia",
  "Implantología",
  "Urgencia",
  "Otro",
];

export function TreatmentsPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken) ?? "";
  const queryClient = useQueryClient();
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState(emptyDraft);
  const editing = Boolean(draft.id);
  const { data = [] } = useQuery({
    queryKey: ["treatments", sessionToken],
    queryFn: () => officeApi.listTreatments(sessionToken),
    enabled: Boolean(sessionToken),
  });
  const mutation = useMutation({
    mutationFn: () => {
      const input = {
        id: draft.id,
        name: draft.name,
        category: draft.category,
        description: draft.description,
        basePriceCents: Math.round(Number(draft.price || 0) * 100),
        estimatedDurationMinutes: draft.duration ? Number(draft.duration) : null,
        requiresFollowUp: draft.requiresFollowUp === "si",
        active: draft.active,
      };
      return editing ? officeApi.updateTreatment(sessionToken, input) : officeApi.createTreatment(sessionToken, input);
    },
    onSuccess: async () => {
      toast.success(editing ? "Tratamiento actualizado" : "Tratamiento guardado");
      setOpen(false);
      setDraft(emptyDraft);
      await queryClient.invalidateQueries({ queryKey: ["treatments"] });
      await queryClient.invalidateQueries({ queryKey: ["reports"] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const submit = () => {
    if (!draft.name.trim()) {
      toast.error("Escribe el nombre del tratamiento");
      return;
    }
    mutation.mutate();
  };

  const editTreatment = (item: TreatmentCatalogItem) => {
    setDraft({
      id: item.id,
      name: item.name,
      category: item.category || "Otro",
      description: item.description ?? "",
      price: item.basePriceCents ? String(item.basePriceCents / 100) : "",
      duration: item.estimatedDurationMinutes ? String(item.estimatedDurationMinutes) : "",
      requiresFollowUp: item.requiresFollowUp ? "si" : "no",
      active: item.active,
    });
    setOpen(true);
  };

  return (
    <div className="space-y-6">
      <PageHeader
        title="Catálogo de tratamientos"
        description="Tratamientos locales editables con categoría, precio base y duración estimada."
        actions={
          <Dialog open={open} onOpenChange={(value) => { setOpen(value); if (!value) setDraft(emptyDraft); }}>
            <DialogTrigger asChild>
              <Button><Plus className="h-4 w-4" />Nuevo</Button>
            </DialogTrigger>
            <DialogContent className="max-h-[92vh] max-w-4xl overflow-y-auto p-0">
              <div className="border-b px-6 py-5">
                <DialogHeader>
                  <DialogTitle>{editing ? "Editar tratamiento" : "Nuevo tratamiento"}</DialogTitle>
                  <DialogDescription>Usa las tarjetas para evitar capturas repetidas y mantener el catálogo limpio.</DialogDescription>
                </DialogHeader>
              </div>
              <form className="grid gap-5 px-6 py-5" onSubmit={(event) => { event.preventDefault(); submit(); }}>
                <div className="grid gap-4 sm:grid-cols-[1fr_180px_180px]">
                  <Field label="Nombre" value={draft.name} onChange={(name) => setDraft({ ...draft, name })} />
                  <Field label="Precio MXN opcional" value={draft.price} onChange={(price) => setDraft({ ...draft, price })} type="number" />
                  <Field label="Duración opcional" value={draft.duration} onChange={(duration) => setDraft({ ...draft, duration })} type="number" />
                </div>
                <Pick label="Categoría" value={draft.category} onChange={(category) => setDraft({ ...draft, category })} items={treatmentCategories.map((category) => ({ value: category, label: category }))} />
                <div className="grid gap-2">
                  <Label>Seguimiento</Label>
                  <SelectableCards
                    value={draft.requiresFollowUp}
                    onChange={(requiresFollowUp) => setDraft({ ...draft, requiresFollowUp })}
                    columns="sm:grid-cols-2"
                    options={[
                      { value: "no", label: "No requiere", description: "Consulta o procedimiento de una sola visita." },
                      { value: "si", label: "Requiere seguimiento", description: "Genera control posterior o varias sesiones." },
                    ]}
                  />
                </div>
                <div className="grid gap-2">
                  <Label>Descripción opcional</Label>
                  <Textarea value={draft.description} onChange={(event) => setDraft({ ...draft, description: event.target.value })} />
                </div>
                <Button type="submit" disabled={mutation.isPending}>{editing ? "Guardar cambios" : "Guardar tratamiento"}</Button>
              </form>
            </DialogContent>
          </Dialog>
        }
      />
      <Card>
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Nombre</TableHead>
                <TableHead>Categoría</TableHead>
                <TableHead>Precio</TableHead>
                <TableHead>Duración</TableHead>
                <TableHead>Seguimiento</TableHead>
                <TableHead className="w-[110px]">Acción</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.map((item) => (
                <TableRow key={item.id}>
                  <TableCell className="font-medium">{item.name}</TableCell>
                  <TableCell>{item.category}</TableCell>
                  <TableCell>{formatCurrency(item.basePriceCents)}</TableCell>
                  <TableCell>{item.estimatedDurationMinutes ? `${item.estimatedDurationMinutes} min` : "Opcional"}</TableCell>
                  <TableCell>{item.requiresFollowUp ? "Sí" : "No"}</TableCell>
                  <TableCell>
                    <Button variant="outline" size="sm" onClick={() => editTreatment(item)}>
                      <Pencil className="h-4 w-4" />Editar
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  );
}

function Field({ label, value, onChange, type = "text" }: { label: string; value: string; onChange: (value: string) => void; type?: string }) {
  return <div className="grid gap-2"><Label>{label}</Label><Input type={type} value={value} onChange={(event) => onChange(event.target.value)} /></div>;
}

function Pick({ label, value, onChange, items }: { label: string; value: string; onChange: (value: string) => void; items: { value: string; label: string }[] }) {
  return (
    <div className="grid gap-2">
      <Label>{label}</Label>
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger><SelectValue placeholder={label} /></SelectTrigger>
        <SelectContent>{items.map((item) => <SelectItem key={item.value} value={item.value}>{item.label}</SelectItem>)}</SelectContent>
      </Select>
    </div>
  );
}
