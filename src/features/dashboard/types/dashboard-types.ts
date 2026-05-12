import type { AppointmentSummary } from "@/features/appointments/types/appointment-types";
import type { AlertSummary, RestockReportItem } from "@/lib/office-api";

export type ChartPoint = {
  label: string;
  value: number;
};

export type DashboardSummary = {
  appointmentsToday: number;
  confirmedToday: number;
  unconfirmedToday: number;
  waitingToday: number;
  revenueTodayCents: number;
  revenueWeekCents: number;
  revenueMonthCents: number;
  pendingEstimates: number;
  approvedEstimates: number;
  newPatientsMonth: number;
  lowInventory: number;
  openAlerts: number;
  activeTreatmentPlans: number;
  upcomingAppointments: AppointmentSummary[];
  incomeSeries: ChartPoint[];
  appointmentStatuses: ChartPoint[];
  paymentMethods: ChartPoint[];
  criticalAlerts: AlertSummary[];
  restockItems: RestockReportItem[];
};
