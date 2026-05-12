import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus } from "lucide-react";
import { toast } from "sonner";
import { PageHeader } from "@/components/data/PageHeader";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from "@/components/ui/sheet";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Textarea } from "@/components/ui/textarea";
import { officeApi } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

export function SuppliersPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken) ?? "";
  const queryClient = useQueryClient();
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState({ name: "", phone: "", email: "", notes: "" });
  const suppliers = useQuery({ queryKey: ["suppliers", sessionToken], queryFn: () => officeApi.listSuppliers(sessionToken), enabled: Boolean(sessionToken) });
  const mutation = useMutation({
    mutationFn: () => officeApi.createSupplier(sessionToken, draft),
    onSuccess: async () => { toast.success("Proveedor guardado"); setOpen(false); await queryClient.invalidateQueries({ queryKey: ["suppliers"] }); },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });
  return <div className="space-y-6"><PageHeader title="Proveedores" description="Directorio local de proveedores dentales." actions={
    <Sheet open={open} onOpenChange={setOpen}><SheetTrigger asChild><Button><Plus className="h-4 w-4" />Nuevo</Button></SheetTrigger><SheetContent><SheetHeader><SheetTitle>Nuevo proveedor</SheetTitle></SheetHeader><div className="mt-6 grid gap-4">
      <Field label="Nombre" value={draft.name} onChange={(name) => setDraft({ ...draft, name })} />
      <Field label="Teléfono" value={draft.phone} onChange={(phone) => setDraft({ ...draft, phone })} />
      <Field label="Correo" value={draft.email} onChange={(email) => setDraft({ ...draft, email })} />
      <div className="grid gap-2"><Label>Notas</Label><Textarea value={draft.notes} onChange={(event) => setDraft({ ...draft, notes: event.target.value })} /></div>
      <Button onClick={() => mutation.mutate()}>Guardar</Button>
    </div></SheetContent></Sheet>} />
    <Card><CardContent className="p-0"><Table><TableHeader><TableRow><TableHead>Nombre</TableHead><TableHead>Teléfono</TableHead><TableHead>Correo</TableHead><TableHead>Notas</TableHead></TableRow></TableHeader><TableBody>{(suppliers.data ?? []).map((supplier) => <TableRow key={supplier.id}><TableCell className="font-medium">{supplier.name}</TableCell><TableCell>{supplier.phone}</TableCell><TableCell>{supplier.email}</TableCell><TableCell>{supplier.notes}</TableCell></TableRow>)}</TableBody></Table></CardContent></Card>
  </div>;
}

function Field({ label, value, onChange }: { label: string; value: string; onChange: (value: string) => void }) {
  return <div className="grid gap-2"><Label>{label}</Label><Input value={value} onChange={(event) => onChange(event.target.value)} /></div>;
}
