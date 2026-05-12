import { z } from "zod";

export const appointmentSchema = z.object({
  patientId: z.string().min(1, "Selecciona un paciente"),
  dentistUserId: z.string().optional(),
  startsAt: z.string().min(1, "Fecha y hora obligatorias"),
  durationMinutes: z.number().min(5).max(480),
  reason: z.string().min(2, "Motivo obligatorio"),
  appointmentType: z.string().min(1, "Tipo obligatorio"),
  notes: z.string().optional(),
});

export type AppointmentFormValues = z.infer<typeof appointmentSchema>;
