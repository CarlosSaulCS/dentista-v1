import { z } from "zod";

export const patientSchema = z.object({
  fullName: z.string().min(2, "Nombre obligatorio"),
  birthDate: z.string().optional(),
  sex: z.string().optional(),
  phone: z.string().optional(),
  whatsapp: z.string().optional(),
  email: z.string().email("Correo inválido").optional().or(z.literal("")),
  address: z.string().optional(),
  emergencyContactName: z.string().optional(),
  emergencyContactPhone: z.string().optional(),
  occupation: z.string().optional(),
  allergies: z.string().optional(),
  systemicDiseases: z.string().optional(),
  currentMedications: z.string().optional(),
  relevantHistory: z.string().optional(),
  habits: z.string().optional(),
  generalNotes: z.string().optional(),
});

export type PatientFormValues = z.infer<typeof patientSchema>;
