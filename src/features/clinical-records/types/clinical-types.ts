export type ClinicalRecordSummary = {
  id: string;
  patientId: string;
  patientName: string;
  responsibleUserId?: string | null;
  responsibleName?: string | null;
  chiefComplaint?: string | null;
  diagnosis?: string | null;
  suggestedPlan?: string | null;
  createdAt: string;
  updatedAt: string;
};

export type ClinicalEvolutionSummary = {
  id: string;
  patientId: string;
  patientName: string;
  responsibleName: string;
  reason: string;
  findings?: string | null;
  proceduresDone?: string | null;
  indications?: string | null;
  nextAppointmentNotes?: string | null;
  createdAt: string;
};
