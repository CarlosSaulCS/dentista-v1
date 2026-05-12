export type PatientSummary = {
  id: string;
  fullName: string;
  birthDate?: string | null;
  sex?: string | null;
  phone?: string | null;
  whatsapp?: string | null;
  email?: string | null;
  allergies?: string | null;
  systemicDiseases?: string | null;
  currentMedications?: string | null;
  status: string;
  createdAt: string;
  updatedAt: string;
};
