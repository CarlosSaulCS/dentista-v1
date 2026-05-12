import { invokeCommand } from "@/lib/api";
import type { ClinicalEvolutionFormValues, ClinicalRecordFormValues } from "@/features/clinical-records/schemas/clinical-schemas";
import type { ClinicalEvolutionSummary, ClinicalRecordSummary } from "@/features/clinical-records/types/clinical-types";

export function createClinicalRecord(sessionToken: string, input: ClinicalRecordFormValues) {
  return invokeCommand<ClinicalRecordSummary>("create_clinical_record", { sessionToken, input });
}

export function createClinicalEvolution(sessionToken: string, input: ClinicalEvolutionFormValues) {
  return invokeCommand<ClinicalEvolutionSummary>("create_clinical_evolution", { sessionToken, input });
}

export function listClinicalRecords(sessionToken: string, patientId: string) {
  return invokeCommand<ClinicalRecordSummary[]>("list_clinical_records", { sessionToken, patientId });
}

export function listClinicalEvolutions(sessionToken: string, patientId: string) {
  return invokeCommand<ClinicalEvolutionSummary[]>("list_clinical_evolutions", { sessionToken, patientId });
}
