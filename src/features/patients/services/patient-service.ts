import { invokeCommand } from "@/lib/api";
import type { PatientFormValues } from "@/features/patients/schemas/patient-schemas";
import type { PatientSummary } from "@/features/patients/types/patient-types";

export function listPatients(sessionToken: string, search = "", limit = 50) {
  return invokeCommand<PatientSummary[]>("list_patients", {
    sessionToken,
    input: { search, limit },
  });
}

export function createPatient(sessionToken: string, input: PatientFormValues) {
  return invokeCommand<PatientSummary>("create_patient", { sessionToken, input });
}
