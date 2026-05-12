import { invokeCommand } from "@/lib/api";
import type { OdontogramRecordView } from "@/features/odontogram/types/odontogram-types";

export function getOdontogram(sessionToken: string, patientId: string, dentitionType: string) {
  return invokeCommand<OdontogramRecordView>("get_odontogram", { sessionToken, patientId, dentitionType });
}

export function upsertOdontogramEntry(
  sessionToken: string,
  input: {
    patientId: string;
    dentitionType: string;
    toothNumber: string;
    surface?: string;
    state: string;
    finding?: string;
  },
) {
  return invokeCommand<OdontogramRecordView>("upsert_odontogram_entry", { sessionToken, input });
}
