export type OdontogramEntry = {
  id: string;
  toothNumber: string;
  surface?: string | null;
  state: string;
  finding?: string | null;
  updatedAt: string;
};

export type OdontogramRecordView = {
  id: string;
  patientId: string;
  dentitionType: string;
  entries: OdontogramEntry[];
};
