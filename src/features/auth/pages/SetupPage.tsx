import { useQueryClient } from "@tanstack/react-query";
import { zodResolver } from "@hookform/resolvers/zod";
import { Building2, ShieldCheck } from "lucide-react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { setupSchema, type SetupFormValues } from "@/features/auth/schemas/auth-schemas";
import { setupClinicAndAdmin } from "@/features/auth/services/auth-service";
import { useAuthStore } from "@/store/auth-store";

export function SetupPage() {
  const setSession = useAuthStore((state) => state.setSession);
  const queryClient = useQueryClient();
  const form = useForm<SetupFormValues>({
    resolver: zodResolver(setupSchema),
    defaultValues: {
      clinicName: "",
      clinicPhone: "",
      clinicWhatsapp: "",
      adminFullName: "",
      adminUsername: "admin",
      adminEmail: "",
      adminPassword: "",
    },
  });

  const onSubmit = form.handleSubmit(async (values) => {
    try {
      const session = await setupClinicAndAdmin(values);
      await queryClient.invalidateQueries({ queryKey: ["bootstrap-status"] });
      setSession(session);
      toast.success("Consultorio y administrador creados");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
      if (message.includes("administrador configurado")) {
        await queryClient.invalidateQueries({ queryKey: ["bootstrap-status"] });
      }
    }
  });

  return (
    <main className="min-h-screen bg-background p-6">
      <div className="mx-auto grid min-h-[calc(100vh-3rem)] max-w-5xl items-center gap-8 lg:grid-cols-[0.9fr_1.1fr]">
        <section className="space-y-5">
          <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary text-primary-foreground">
            <ShieldCheck className="h-6 w-6" />
          </div>
          <div>
            <h1 className="text-3xl font-semibold tracking-tight">Dentista v1 Professional</h1>
            <p className="mt-3 max-w-xl text-base text-muted-foreground">
              Inicializa el consultorio local, crea el administrador y deja la base segura para operar sin internet.
            </p>
          </div>
        </section>

        <Card className="border-border shadow-sm">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Building2 className="h-5 w-5 text-primary" />
              Configuración inicial
            </CardTitle>
            <CardDescription>Este asistente solo aparece antes de crear el primer usuario.</CardDescription>
          </CardHeader>
          <CardContent>
            <form className="grid gap-4" onSubmit={onSubmit}>
              <div className="grid gap-2">
                <Label htmlFor="clinicName">Nombre del consultorio</Label>
                <Input id="clinicName" {...form.register("clinicName")} autoFocus />
                <FieldError message={form.formState.errors.clinicName?.message} />
              </div>
              <div className="grid gap-4 sm:grid-cols-2">
                <div className="grid gap-2">
                  <Label htmlFor="clinicPhone">Teléfono</Label>
                  <Input id="clinicPhone" {...form.register("clinicPhone")} />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="clinicWhatsapp">WhatsApp</Label>
                  <Input id="clinicWhatsapp" {...form.register("clinicWhatsapp")} />
                </div>
              </div>
              <div className="grid gap-4 sm:grid-cols-2">
                <div className="grid gap-2">
                  <Label htmlFor="adminFullName">Administrador</Label>
                  <Input id="adminFullName" {...form.register("adminFullName")} />
                  <FieldError message={form.formState.errors.adminFullName?.message} />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="adminUsername">Usuario</Label>
                  <Input id="adminUsername" {...form.register("adminUsername")} />
                  <FieldError message={form.formState.errors.adminUsername?.message} />
                </div>
              </div>
              <div className="grid gap-4 sm:grid-cols-2">
                <div className="grid gap-2">
                  <Label htmlFor="adminEmail">Correo</Label>
                  <Input id="adminEmail" type="email" {...form.register("adminEmail")} />
                  <FieldError message={form.formState.errors.adminEmail?.message} />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="adminPassword">Contraseña</Label>
                  <Input id="adminPassword" type="password" {...form.register("adminPassword")} />
                  <FieldError message={form.formState.errors.adminPassword?.message} />
                </div>
              </div>
              <Button type="submit" disabled={form.formState.isSubmitting}>
                Crear sistema local
              </Button>
            </form>
          </CardContent>
        </Card>
      </div>
    </main>
  );
}

function FieldError({ message }: { message?: string }) {
  if (!message) return null;
  return <p className="text-xs text-destructive">{message}</p>;
}
