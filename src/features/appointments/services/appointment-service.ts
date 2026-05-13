import { invokeCommand } from "@/lib/api";
import type { AppointmentFormValues } from "@/features/appointments/schemas/appointment-schemas";
import type { AppointmentSummary } from "@/features/appointments/types/appointment-types";

export function listAppointments(sessionToken: string, date: string, status = "") {
  return invokeCommand<AppointmentSummary[]>("list_appointments", {
    sessionToken,
    input: { date, status },
  });
}

export function getNextPatientAppointment(sessionToken: string, patientId: string) {
  return invokeCommand<AppointmentSummary | null>("get_next_patient_appointment", {
    sessionToken,
    patientId,
  });
}

export function createAppointment(sessionToken: string, input: AppointmentFormValues) {
  return invokeCommand<AppointmentSummary>("create_appointment", { sessionToken, input });
}

export function updateAppointment(sessionToken: string, input: AppointmentFormValues & { id: string; status: string }) {
  return invokeCommand<AppointmentSummary>("update_appointment", { sessionToken, input });
}

export function updateAppointmentStatus(sessionToken: string, appointmentId: string, status: string, notes?: string) {
  return invokeCommand<AppointmentSummary>("update_appointment_status", {
    sessionToken,
    input: { appointmentId, status, notes },
  });
}

export function softDeleteAppointment(sessionToken: string, appointmentId: string) {
  return invokeCommand<AppointmentSummary>("soft_delete_appointment", { sessionToken, appointmentId });
}
