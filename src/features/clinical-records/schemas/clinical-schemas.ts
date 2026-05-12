import { z } from "zod";

export const clinicalRecordSchema = z.object({
  patientId: z.string().min(1, "Selecciona paciente"),
  chiefComplaint: z.string().optional(),
  currentCondition: z.string().optional(),
  hereditaryHistory: z.string().optional(),
  pathologicalHistory: z.string().optional(),
  nonPathologicalHistory: z.string().optional(),
  allergies: z.string().optional(),
  currentMedications: z.string().optional(),
  systemicDiseases: z.string().optional(),
  habits: z.string().optional(),
  clinicalExploration: z.string().optional(),
  diagnosis: z.string().optional(),
  prognosis: z.string().optional(),
  suggestedPlan: z.string().optional(),
  indications: z.string().optional(),
  observations: z.string().optional(),
});

export const clinicalEvolutionSchema = z.object({
  patientId: z.string().min(1, "Selecciona paciente"),
  clinicalRecordId: z.string().optional(),
  reason: z.string().min(2, "Motivo obligatorio"),
  findings: z.string().optional(),
  proceduresDone: z.string().optional(),
  indications: z.string().optional(),
  nextAppointmentNotes: z.string().optional(),
});

export type ClinicalRecordFormValues = z.infer<typeof clinicalRecordSchema>;
export type ClinicalEvolutionFormValues = z.infer<typeof clinicalEvolutionSchema>;
