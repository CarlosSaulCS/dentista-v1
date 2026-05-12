import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus } from "lucide-react";
import { toast } from "sonner";
import { PageHeader } from "@/components/data/PageHeader";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from "@/components/ui/sheet";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { listUsers } from "@/features/auth/services/auth-service";
import { officeApi } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

export function UsersPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken) ?? "";
  const queryClient = useQueryClient();
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState({ fullName: "", username: "", email: "", roleId: "", password: "", professionalLicense: "", specialty: "" });
  const users = useQuery({ queryKey: ["users", sessionToken], queryFn: () => listUsers(sessionToken), enabled: Boolean(sessionToken) });
  const roles = useQuery({ queryKey: ["roles", sessionToken], queryFn: () => officeApi.listRoles(sessionToken), enabled: Boolean(sessionToken) });
  const mutation = useMutation({ mutationFn: () => officeApi.createUser(sessionToken, draft), onSuccess: async () => { toast.success("Usuario creado"); setOpen(false); await queryClient.invalidateQueries({ queryKey: ["users"] }); }, onError: (error) => toast.error(error instanceof Error ? error.message : String(error)) });
  return <div className="space-y-6"><PageHeader title="Usuarios y permisos" description="Alta local de usuarios por rol." actions={
    <Sheet open={open} onOpenChange={setOpen}><SheetTrigger asChild><Button><Plus className="h-4 w-4" />Nuevo usuario</Button></SheetTrigger><SheetContent><SheetHeader><SheetTitle>Nuevo usuario</SheetTitle></SheetHeader><div className="mt-6 grid gap-4">
      <Field label="Nombre completo" value={draft.fullName} onChange={(fullName) => setDraft({ ...draft, fullName })} />
      <Field label="Usuario" value={draft.username} onChange={(username) => setDraft({ ...draft, username })} />
      <Field label="Correo" value={draft.email} onChange={(email) => setDraft({ ...draft, email })} />
      <div className="grid gap-2"><Label>Rol</Label><Select value={draft.roleId} onValueChange={(roleId) => setDraft({ ...draft, roleId })}><SelectTrigger><SelectValue placeholder="Rol" /></SelectTrigger><SelectContent>{(roles.data ?? []).map((role) => <SelectItem key={role.id} value={role.id}>{role.name}</SelectItem>)}</SelectContent></Select></div>
      <Field label="Contraseña" value={draft.password} onChange={(password) => setDraft({ ...draft, password })} type="password" />
      <Field label="Cédula profesional" value={draft.professionalLicense} onChange={(professionalLicense) => setDraft({ ...draft, professionalLicense })} />
      <Field label="Especialidad" value={draft.specialty} onChange={(specialty) => setDraft({ ...draft, specialty })} />
      <Button onClick={() => mutation.mutate()}>Crear usuario</Button>
    </div></SheetContent></Sheet>} />
    <Card><CardContent className="p-0"><Table><TableHeader><TableRow><TableHead>Nombre</TableHead><TableHead>Usuario</TableHead><TableHead>Rol</TableHead></TableRow></TableHeader><TableBody>{(users.data ?? []).map((user) => <TableRow key={user.id}><TableCell className="font-medium">{user.fullName}</TableCell><TableCell>{user.username}</TableCell><TableCell>{user.roleName}</TableCell></TableRow>)}</TableBody></Table></CardContent></Card>
  </div>;
}

function Field({ label, value, onChange, type = "text" }: { label: string; value: string; onChange: (value: string) => void; type?: string }) {
  return <div className="grid gap-2"><Label>{label}</Label><Input type={type} value={value} onChange={(event) => onChange(event.target.value)} /></div>;
}
