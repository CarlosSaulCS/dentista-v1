import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Edit, Plus, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { PageHeader } from "@/components/data/PageHeader";
import { SelectableCards } from "@/components/data/SelectableCards";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { formatCurrency } from "@/lib/api";
import { officeApi, type InventoryItemSummary } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

const emptyDraft = {
  name: "",
  category: "Consumibles",
  customCategory: "",
  unit: "pieza",
  quantity: "",
  minimum: "",
  cost: "",
  expirationMode: "no",
  expiration: "",
  location: "",
  supplierId: "none",
};

const inventoryCategories = [
  "Consumibles",
  "Material dental",
  "Instrumental",
  "Medicamentos",
  "Esterilización",
  "Limpieza",
  "Ortodoncia",
  "Otro",
];

const units = ["pieza", "caja", "paquete", "par", "ml", "g", "rollo", "bolsa"];

export function InventoryPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken) ?? "";
  const queryClient = useQueryClient();
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState(emptyDraft);
  const [editingItem, setEditingItem] = useState<InventoryItemSummary | null>(null);
  const [movement, setMovement] = useState({ inventoryItemId: "", movementType: "entrada", quantity: "", reason: "" });
  const inventory = useQuery({ queryKey: ["inventory", sessionToken], queryFn: () => officeApi.listInventoryItems(sessionToken), enabled: Boolean(sessionToken) });
  const suppliers = useQuery({ queryKey: ["suppliers", sessionToken], queryFn: () => officeApi.listSuppliers(sessionToken), enabled: Boolean(sessionToken) });
  const mutation = useMutation({
    mutationFn: () => {
      const input = {
        supplierId: draft.supplierId === "none" ? "" : draft.supplierId,
        name: draft.name,
        category: effectiveCategory(draft),
        unit: draft.unit,
        currentQuantity: Number(draft.quantity || 0),
        minimumStock: Number(draft.minimum || 0),
        costCents: Math.round(Number(draft.cost || 0) * 100),
        purchaseDate: "",
        expirationDate: draft.expirationMode === "si" ? draft.expiration : "",
        location: draft.location,
        active: true,
      };
      return editingItem
        ? officeApi.updateInventoryItem(sessionToken, { ...input, id: editingItem.id })
        : officeApi.createInventoryItem(sessionToken, input);
    },
    onSuccess: async () => {
      toast.success(editingItem ? "Insumo actualizado" : "Insumo guardado");
      setOpen(false);
      setDraft(emptyDraft);
      setEditingItem(null);
      await invalidateOperationalData(queryClient);
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });
  const movementMutation = useMutation({
    mutationFn: () => officeApi.createInventoryMovement(sessionToken, { inventoryItemId: movement.inventoryItemId, movementType: movement.movementType, quantity: Number(movement.quantity || 0), reason: movement.reason }),
    onSuccess: async () => {
      toast.success("Movimiento registrado");
      setMovement({ inventoryItemId: "", movementType: "entrada", quantity: "", reason: "" });
      await invalidateOperationalData(queryClient);
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });
  const deleteMutation = useMutation({
    mutationFn: (inventoryItemId: string) => officeApi.softDeleteInventoryItem(sessionToken, inventoryItemId),
    onSuccess: async () => {
      toast.success("Insumo dado de baja");
      await invalidateOperationalData(queryClient);
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const submitItem = () => {
    const quantity = Number(draft.quantity || 0);
    const minimum = Number(draft.minimum || 0);
    const cost = Number(draft.cost || 0);
    if (!draft.name.trim()) {
      toast.error("Escribe el nombre del insumo");
      return;
    }
    if (!effectiveCategory(draft).trim()) {
      toast.error("Selecciona o escribe una categoría");
      return;
    }
    if (!Number.isFinite(quantity) || quantity < 0) {
      toast.error("La cantidad actual no puede ser negativa");
      return;
    }
    if (!Number.isFinite(minimum) || minimum < 0) {
      toast.error("El stock mínimo no puede ser negativo");
      return;
    }
    if (!Number.isFinite(cost) || cost < 0) {
      toast.error("El costo no puede ser negativo");
      return;
    }
    if (draft.expirationMode === "si" && !draft.expiration) {
      toast.error("Captura la fecha de caducidad");
      return;
    }
    mutation.mutate();
  };

  const submitMovement = () => {
    const quantity = Number(movement.quantity);
    if (!movement.inventoryItemId) {
      toast.error("Selecciona un insumo");
      return;
    }
    if (!Number.isFinite(quantity) || quantity <= 0) {
      toast.error("La cantidad del movimiento debe ser mayor a cero");
      return;
    }
    movementMutation.mutate();
  };

  return <div className="space-y-6">
    <PageHeader title="Inventario dental" description="Control local de insumos, stock mínimo y caducidad opcional." actions={
      <Dialog open={open} onOpenChange={(value) => { setOpen(value); if (!value) { setDraft(emptyDraft); setEditingItem(null); } }}>
        <DialogTrigger asChild><Button><Plus className="h-4 w-4" />Nuevo insumo</Button></DialogTrigger>
        <DialogContent className="max-h-[92vh] max-w-4xl overflow-y-auto p-0">
          <div className="border-b px-6 py-5">
            <DialogHeader>
              <DialogTitle>{editingItem ? "Editar insumo" : "Nuevo insumo"}</DialogTitle>
              <DialogDescription>Registra solo lo necesario; proveedor, costo, ubicación y caducidad pueden quedar vacíos.</DialogDescription>
            </DialogHeader>
          </div>
          <form className="grid gap-5 px-6 py-5" onSubmit={(event) => { event.preventDefault(); submitItem(); }}>
            <div className="grid gap-4 sm:grid-cols-[1fr_220px]">
              <Field label="Nombre" value={draft.name} onChange={(name) => setDraft({ ...draft, name })} />
              <Pick
                label="Proveedor opcional"
                value={draft.supplierId}
                onChange={(supplierId) => setDraft({ ...draft, supplierId })}
                items={[{ value: "none", label: "Sin proveedor" }, ...(suppliers.data ?? []).map((s) => ({ value: s.id, label: s.name }))]}
              />
            </div>
            <div className="grid gap-2">
              <Pick label="Categoría" value={draft.category} onChange={(category) => setDraft({ ...draft, category })} items={inventoryCategories.map((category) => ({ value: category, label: category }))} />
            </div>
            {draft.category === "Otro" ? (
              <Field label="Otra categoría" value={draft.customCategory} onChange={(customCategory) => setDraft({ ...draft, customCategory })} />
            ) : null}
            <div className="grid gap-2">
              <Pick label="Unidad" value={draft.unit} onChange={(unit) => setDraft({ ...draft, unit })} items={units.map((unit) => ({ value: unit, label: unit }))} />
            </div>
            <div className="grid gap-4 sm:grid-cols-4">
              <Field label="Cantidad actual" value={draft.quantity} onChange={(quantity) => setDraft({ ...draft, quantity })} type="number" />
              <Field label="Stock mínimo" value={draft.minimum} onChange={(minimum) => setDraft({ ...draft, minimum })} type="number" />
              <Field label="Costo MXN opcional" value={draft.cost} onChange={(cost) => setDraft({ ...draft, cost })} type="number" />
              <Field label="Ubicación opcional" value={draft.location} onChange={(location) => setDraft({ ...draft, location })} />
            </div>
            <div className="grid gap-2">
              <Label>Caducidad</Label>
              <SelectableCards
                value={draft.expirationMode}
                onChange={(expirationMode) => setDraft({ ...draft, expirationMode, expiration: expirationMode === "no" ? "" : draft.expiration })}
                columns="sm:grid-cols-2"
                options={[
                  { value: "no", label: "No caduca", description: "Guantes, ligas, instrumental u otros insumos sin fecha." },
                  { value: "si", label: "Tiene caducidad", description: "Medicamentos, materiales o productos con fecha de vencimiento." },
                ]}
              />
            </div>
            {draft.expirationMode === "si" ? (
              <Field label="Fecha de caducidad" value={draft.expiration} onChange={(expiration) => setDraft({ ...draft, expiration })} type="date" />
            ) : null}
            <Button type="submit" disabled={mutation.isPending}>{editingItem ? "Actualizar insumo" : "Guardar insumo"}</Button>
          </form>
        </DialogContent>
      </Dialog>
    } />
    <Card>
      <CardContent className="grid gap-4 p-4 md:grid-cols-[1fr_1fr_160px_1fr_160px] md:items-end">
        <Pick label="Insumo" value={movement.inventoryItemId} onChange={(inventoryItemId) => setMovement({ ...movement, inventoryItemId })} items={(inventory.data ?? []).map((item) => ({ value: item.id, label: item.name }))} />
        <div className="grid gap-2">
          <Pick
            label="Movimiento"
            value={movement.movementType}
            onChange={(movementType) => setMovement({ ...movement, movementType })}
            items={[
              { value: "entrada", label: "Entrada" },
              { value: "salida", label: "Salida" },
              { value: "ajuste", label: "Ajuste" },
              { value: "consumo", label: "Consumo" },
              { value: "merma", label: "Merma" },
            ]}
          />
        </div>
        <Field label="Cantidad" value={movement.quantity} onChange={(quantity) => setMovement({ ...movement, quantity })} type="number" />
        <Field label="Motivo opcional" value={movement.reason} onChange={(reason) => setMovement({ ...movement, reason })} />
        <Button onClick={submitMovement} disabled={movementMutation.isPending}>Registrar</Button>
      </CardContent>
    </Card>
    <Card><CardContent className="overflow-x-auto p-0"><Table>
      <TableHeader><TableRow><TableHead>Insumo</TableHead><TableHead>Categoría</TableHead><TableHead>Stock</TableHead><TableHead>Costo</TableHead><TableHead>Proveedor</TableHead><TableHead>Caducidad</TableHead><TableHead>Acciones</TableHead></TableRow></TableHeader>
      <TableBody>{(inventory.data ?? []).length === 0 ? (
        <TableRow><TableCell colSpan={7} className="h-24 text-center text-muted-foreground">No hay insumos registrados.</TableCell></TableRow>
      ) : (inventory.data ?? []).map((item) => <TableRow key={item.id} className={item.currentQuantity <= item.minimumStock ? "bg-amber-50" : ""}>
        <TableCell className="font-medium">{item.name}</TableCell><TableCell>{item.category}</TableCell><TableCell>{item.currentQuantity} / min {item.minimumStock} {item.unit}</TableCell><TableCell>{formatCurrency(item.costCents)}</TableCell><TableCell>{item.supplierName ?? "Sin proveedor"}</TableCell><TableCell>{item.expirationDate ?? "No aplica"}</TableCell>
        <TableCell>
          <div className="flex flex-wrap gap-2">
            <Button variant="outline" size="sm" onClick={() => { setEditingItem(item); setDraft(draftFromItem(item)); setOpen(true); }}>
              <Edit className="h-4 w-4" />
              Editar
            </Button>
            <AlertDialog>
              <AlertDialogTrigger asChild>
                <Button variant="outline" size="sm">
                  <Trash2 className="h-4 w-4" />
                  Baja
                </Button>
              </AlertDialogTrigger>
              <AlertDialogContent>
                <AlertDialogHeader>
                  <AlertDialogTitle>Dar de baja {item.name}</AlertDialogTitle>
                  <AlertDialogDescription>
                    El insumo quedará fuera de inventario activo sin borrarse físicamente.
                  </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                  <AlertDialogCancel>Cancelar</AlertDialogCancel>
                  <AlertDialogAction onClick={() => deleteMutation.mutate(item.id)}>Confirmar baja</AlertDialogAction>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
          </div>
        </TableCell>
      </TableRow>)}</TableBody>
    </Table></CardContent></Card>
  </div>;
}

function effectiveCategory(draft: typeof emptyDraft) {
  return draft.category === "Otro" ? draft.customCategory : draft.category;
}

function draftFromItem(item: InventoryItemSummary): typeof emptyDraft {
  const knownCategory = inventoryCategories.includes(item.category);
  return {
    name: item.name,
    category: knownCategory ? item.category : "Otro",
    customCategory: knownCategory ? "" : item.category,
    unit: item.unit,
    quantity: String(item.currentQuantity),
    minimum: String(item.minimumStock),
    cost: String(item.costCents / 100),
    expirationMode: item.expirationDate ? "si" : "no",
    expiration: item.expirationDate ?? "",
    location: item.location ?? "",
    supplierId: item.supplierId ?? "none",
  };
}

async function invalidateOperationalData(queryClient: ReturnType<typeof useQueryClient>) {
  await queryClient.invalidateQueries({ queryKey: ["inventory"] });
  await queryClient.invalidateQueries({ queryKey: ["alerts"] });
  await queryClient.invalidateQueries({ queryKey: ["dashboard-summary"] });
  await queryClient.invalidateQueries({ queryKey: ["reports"] });
}

function Field({ label, value, onChange, type = "text" }: { label: string; value: string; onChange: (value: string) => void; type?: string }) {
  return <div className="grid gap-2"><Label>{label}</Label><Input type={type} value={value} onChange={(event) => onChange(event.target.value)} /></div>;
}

function Pick({ label, value, onChange, items }: { label: string; value: string; onChange: (value: string) => void; items: { value: string; label: string }[] }) {
  return <div className="grid gap-2"><Label>{label}</Label><Select value={value} onValueChange={onChange}><SelectTrigger><SelectValue placeholder={label} /></SelectTrigger><SelectContent>{items.map((item) => <SelectItem key={item.value} value={item.value}>{item.label}</SelectItem>)}</SelectContent></Select></div>;
}
