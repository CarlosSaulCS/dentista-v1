import { z } from "zod";

export const setupSchema = z.object({
  clinicName: z.string().min(2, "Nombre del consultorio obligatorio"),
  clinicPhone: z.string().optional(),
  clinicWhatsapp: z.string().optional(),
  adminFullName: z.string().min(2, "Nombre del administrador obligatorio"),
  adminUsername: z.string().min(3, "Usuario mínimo de 3 caracteres"),
  adminEmail: z.string().email("Correo inválido").optional().or(z.literal("")),
  adminPassword: z.string().min(8, "La contraseña requiere al menos 8 caracteres"),
});

export const loginSchema = z.object({
  username: z.string().min(1, "Usuario obligatorio"),
  password: z.string().min(1, "Contraseña obligatoria"),
});

export type SetupFormValues = z.infer<typeof setupSchema>;
export type LoginFormValues = z.infer<typeof loginSchema>;
