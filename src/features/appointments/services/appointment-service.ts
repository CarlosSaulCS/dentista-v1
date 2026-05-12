import { invokeCommand } from "@/lib/api";
import type { AppointmentFormValues } from "@/features/appointments/schemas/appointment-schemas";
import type { AppointmentSummary } from "@/features/appointments/types/appointment-types";

export function listAppointments(sessionToken: string, date: string, status = "") {
  return invokeCommand<AppointmentSummary[]>("list_appointments", {
    sessionToken,
    input: { date, status },
  });
}

export function createAppointment(sessionToken: string, input: AppointmentFormValues) {
  return invokeCommand<AppointmentSummary>("create_appointment", { sessionToken, input });
}

export function updateAppointmentStatus(sessionToken: string, appointmentId: string, status: string, notes?: string) {
  return invokeCommand<AppointmentSummary>("update_appointment_status", {
    sessionToken,
    input: { appointmentId, status, notes },
  });
}
