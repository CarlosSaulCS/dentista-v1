import { invokeCommand } from "@/lib/api";
import type { DashboardSummary } from "@/features/dashboard/types/dashboard-types";

export function getDashboardSummary(sessionToken: string) {
  return invokeCommand<DashboardSummary>("get_dashboard_summary", { sessionToken });
}
