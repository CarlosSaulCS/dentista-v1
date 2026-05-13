import { useQueryClient } from "@tanstack/react-query";
import { zodResolver } from "@hookform/resolvers/zod";
import { LockKeyhole } from "lucide-react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { loginSchema, type LoginFormValues } from "@/features/auth/schemas/auth-schemas";
import { login } from "@/features/auth/services/auth-service";
import { useAuthStore } from "@/store/auth-store";
import type { BootstrapStatus, LicenseStatus } from "@/types/shared";

export function LoginPage({ license }: { license?: LicenseStatus }) {
  const setSession = useAuthStore((state) => state.setSession);
  const locked = useAuthStore((state) => state.locked);
  const queryClient = useQueryClient();
  const form = useForm<LoginFormValues>({
    resolver: zodResolver(loginSchema),
    defaultValues: { username: "", password: "" },
  });

  const onSubmit = form.handleSubmit(async (values) => {
    try {
      const session = await login(values);
      queryClient.setQueryData<BootstrapStatus>(["bootstrap-status"], (current) =>
        current ? { ...current, license: session.license } : current,
      );
      setSession(session);
      toast.success("Sesión iniciada");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  });

  return (
    <main className="grid min-h-screen place-items-center bg-background p-6">
      <Card className="w-full max-w-md border-border shadow-sm">
        <CardHeader className="space-y-3 text-center">
          <div className="mx-auto flex h-11 w-11 items-center justify-center rounded-lg bg-primary text-primary-foreground">
            <LockKeyhole className="h-5 w-5" />
          </div>
          <div>
            <CardTitle>DentalCare Manager</CardTitle>
            <CardDescription>
              {locked ? "Sesión bloqueada por inactividad" : licenseDescription(license)}
            </CardDescription>
          </div>
        </CardHeader>
        <CardContent>
          <form className="grid gap-4" onSubmit={onSubmit}>
            <div className="grid gap-2">
              <Label htmlFor="username">Usuario o correo</Label>
              <Input id="username" {...form.register("username")} autoFocus />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="password">Contraseña</Label>
              <Input id="password" type="password" {...form.register("password")} />
            </div>
            <Button type="submit" disabled={form.formState.isSubmitting}>
              Entrar
            </Button>
            {license?.requiresActivation ? (
              <div className="rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-sm text-destructive">
                La prueba terminó. Ingresa tu usuario y la clave de activación en el campo contraseña para habilitar el sistema.
              </div>
            ) : null}
          </form>
        </CardContent>
      </Card>
    </main>
  );
}

function licenseDescription(license?: LicenseStatus) {
  if (!license) return "Sistema Integral para Consultorio Dental";
  if (license.isLicensed) return "Sistema activado para operación completa";
  if (license.requiresActivation) return "Prueba de 30 días finalizada";
  if (license.isTrialActive) {
    return `Prueba activa: ${license.daysRemaining} día${license.daysRemaining === 1 ? "" : "s"} restante${license.daysRemaining === 1 ? "" : "s"}`;
  }
  return "Sistema Integral para Consultorio Dental";
}
