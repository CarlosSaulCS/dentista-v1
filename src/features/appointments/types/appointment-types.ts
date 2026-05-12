export type AppointmentSummary = {
  id: string;
  patientId: string;
  patientName: string;
  dentistUserId?: string | null;
  dentistName?: string | null;
  startsAt: string;
  endsAt: string;
  durationMinutes: number;
  reason: string;
  appointmentType: string;
  status: string;
  notes?: string | null;
};
