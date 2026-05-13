import { invokeCommand } from "@/lib/api";
import type { ClinicSummary, UserListItem } from "@/types/shared";

export type TreatmentCatalogItem = {
  id: string;
  name: string;
  category: string;
  description?: string | null;
  basePriceCents: number;
  estimatedDurationMinutes?: number | null;
  requiresFollowUp: boolean;
  active: boolean;
};

export type TreatmentPlanSummary = {
  id: string;
  patientId: string;
  patientName: string;
  diagnosis?: string | null;
  subtotalCents: number;
  discountCents: number;
  totalCents: number;
  paidCents: number;
  balanceCents: number;
  status: string;
  notes?: string | null;
  createdAt: string;
};

export type TreatmentPlanItemView = {
  id: string;
  treatmentPlanId: string;
  treatmentName?: string | null;
  toothNumber?: string | null;
  diagnosis?: string | null;
  phase?: string | null;
  priority?: string | null;
  quantity: number;
  unitPriceCents: number;
  discountCents: number;
  totalCents: number;
  status: string;
  notes?: string | null;
};

export type EstimateSummary = {
  id: string;
  patientId: string;
  patientName: string;
  treatmentPlanId?: string | null;
  folio: string;
  status: string;
  validUntil?: string | null;
  subtotalCents: number;
  discountCents: number;
  totalCents: number;
  observations?: string | null;
  terms?: string | null;
  createdAt: string;
};

export type EstimateItemView = {
  id: string;
  estimateId: string;
  description: string;
  quantity: number;
  unitPriceCents: number;
  discountCents: number;
  totalCents: number;
};

export type PaymentSummary = {
  id: string;
  patientId: string;
  patientName: string;
  folio: string;
  concept: string;
  amountCents: number;
  method: string;
  status: string;
  paidAt: string;
  receivedByName: string;
  notes?: string | null;
  proofFilesCount: number;
};

export type CashRegisterSummary = {
  id: string;
  openedByName: string;
  openedAt: string;
  openingFloatCents: number;
  status: string;
  closedAt?: string | null;
  totalCashCents: number;
  totalTransferCents: number;
  totalCardCents: number;
  totalOtherCents: number;
};

export type SupplierSummary = {
  id: string;
  name: string;
  phone?: string | null;
  email?: string | null;
  notes?: string | null;
  active: boolean;
};

export type InventoryItemSummary = {
  id: string;
  supplierId?: string | null;
  supplierName?: string | null;
  name: string;
  category: string;
  unit: string;
  currentQuantity: number;
  minimumStock: number;
  costCents: number;
  expirationDate?: string | null;
  location?: string | null;
  active: boolean;
};

export type RestockReportItem = {
  id: string;
  name: string;
  category: string;
  unit: string;
  currentQuantity: number;
  minimumStock: number;
  suggestedQuantity: number;
  costCents: number;
  estimatedCostCents: number;
  supplierName?: string | null;
  expirationDate?: string | null;
  location?: string | null;
};

export type AlertSummary = {
  id: string;
  patientName?: string | null;
  alertType: string;
  priority: string;
  title: string;
  message: string;
  dueAt?: string | null;
  status: string;
  createdAt: string;
};

export type PatientFileSummary = {
  id: string;
  patientId?: string | null;
  patientName?: string | null;
  categoryName?: string | null;
  fileType: string;
  originalName: string;
  relativePath: string;
  mimeType?: string | null;
  sizeBytes: number;
  description?: string | null;
  createdAt: string;
};

export type ConsentTemplateSummary = {
  id: string;
  name: string;
  treatmentCategory?: string | null;
  body: string;
  active: boolean;
};

export type ReportsSummary = {
  incomeCents: number;
  paymentsCount: number;
  appointmentsCount: number;
  cancelledAppointments: number;
  newPatients: number;
  estimatesTotal: number;
  estimatesApproved: number;
  pendingBalancesCents: number;
  lowInventory: number;
  restockItems: RestockReportItem[];
  incomeByMethod: { label: string; value: number }[];
  appointmentsByStatus: { label: string; value: number }[];
};

export type ReportExportResult = {
  id: string;
  path: string;
  sizeBytes: number;
  createdAt: string;
};

export type GlobalSearchResult = {
  entityType: string;
  id: string;
  title: string;
  subtitle?: string | null;
  route: string;
  status?: string | null;
};

export type MessageTemplateSummary = {
  id: string;
  name: string;
  body: string;
};

export type RoleSummary = {
  id: string;
  name: string;
  systemKey: string;
};

export type PeriodontalRecordSummary = {
  id: string;
  patientId: string;
  patientName: string;
  status: string;
  notes?: string | null;
  createdAt: string;
  updatedAt: string;
};

export const officeApi = {
  listTreatments: (sessionToken: string) => invokeCommand<TreatmentCatalogItem[]>("list_treatments", { sessionToken }),
  createTreatment: (sessionToken: string, input: Record<string, unknown>) =>
    invokeCommand<TreatmentCatalogItem>("create_treatment", { sessionToken, input }),
  updateTreatment: (sessionToken: string, input: Record<string, unknown>) =>
    invokeCommand<TreatmentCatalogItem>("update_treatment", { sessionToken, input }),
  listTreatmentPlans: (sessionToken: string) =>
    invokeCommand<TreatmentPlanSummary[]>("list_treatment_plans", { sessionToken }),
  createTreatmentPlan: (sessionToken: string, input: Record<string, unknown>) =>
    invokeCommand<TreatmentPlanSummary>("create_treatment_plan", { sessionToken, input }),
  listTreatmentPlanItems: (sessionToken: string, treatmentPlanId: string) =>
    invokeCommand<TreatmentPlanItemView[]>("list_treatment_plan_items", { sessionToken, treatmentPlanId }),
  listEstimates: (sessionToken: string) => invokeCommand<EstimateSummary[]>("list_estimates", { sessionToken }),
  createEstimate: (sessionToken: string, input: Record<string, unknown>) =>
    invokeCommand<EstimateSummary>("create_estimate", { sessionToken, input }),
  updateEstimateStatus: (sessionToken: string, id: string, status: string) =>
    invokeCommand<EstimateSummary>("update_estimate_status", { sessionToken, input: { id, status } }),
  listEstimateItems: (sessionToken: string, estimateId: string) =>
    invokeCommand<EstimateItemView[]>("list_estimate_items", { sessionToken, estimateId }),
  listPayments: (sessionToken: string) => invokeCommand<PaymentSummary[]>("list_payments", { sessionToken }),
  registerPayment: (sessionToken: string, input: Record<string, unknown>) =>
    invokeCommand<PaymentSummary>("register_payment", { sessionToken, input }),
  getCurrentCashRegister: (sessionToken: string) =>
    invokeCommand<CashRegisterSummary | null>("get_current_cash_register", { sessionToken }),
  openCashRegister: (sessionToken: string, openingFloatCents: number) =>
    invokeCommand<CashRegisterSummary>("open_cash_register", { sessionToken, input: { openingFloatCents } }),
  closeCashRegister: (sessionToken: string, cashRegisterId: string, countedCashCents: number) =>
    invokeCommand("close_cash_register", { sessionToken, input: { cashRegisterId, countedCashCents } }),
  listSuppliers: (sessionToken: string) => invokeCommand<SupplierSummary[]>("list_suppliers", { sessionToken }),
  createSupplier: (sessionToken: string, input: Record<string, unknown>) =>
    invokeCommand<SupplierSummary>("create_supplier", { sessionToken, input }),
  listInventoryItems: (sessionToken: string) =>
    invokeCommand<InventoryItemSummary[]>("list_inventory_items", { sessionToken }),
  createInventoryItem: (sessionToken: string, input: Record<string, unknown>) =>
    invokeCommand<InventoryItemSummary>("create_inventory_item", { sessionToken, input }),
  updateInventoryItem: (sessionToken: string, input: Record<string, unknown>) =>
    invokeCommand<InventoryItemSummary>("update_inventory_item", { sessionToken, input }),
  softDeleteInventoryItem: (sessionToken: string, inventoryItemId: string) =>
    invokeCommand<InventoryItemSummary>("soft_delete_inventory_item", { sessionToken, inventoryItemId }),
  createInventoryMovement: (sessionToken: string, input: Record<string, unknown>) =>
    invokeCommand<InventoryItemSummary>("create_inventory_movement", { sessionToken, input }),
  listAlerts: (sessionToken: string) => invokeCommand<AlertSummary[]>("list_alerts", { sessionToken }),
  createAlert: (sessionToken: string, input: Record<string, unknown>) =>
    invokeCommand<AlertSummary>("create_alert", { sessionToken, input }),
  resolveAlert: (sessionToken: string, id: string) =>
    invokeCommand<AlertSummary>("resolve_alert", { sessionToken, id }),
  savePatientFile: (sessionToken: string, input: Record<string, unknown>) =>
    invokeCommand<PatientFileSummary>("save_patient_file", { sessionToken, input }),
  listPatientFiles: (sessionToken: string) => invokeCommand<PatientFileSummary[]>("list_patient_files", { sessionToken }),
  openPatientFile: (sessionToken: string, fileId: string) =>
    invokeCommand<void>("open_patient_file", { sessionToken, fileId }),
  openExternalUrl: (sessionToken: string, url: string) =>
    invokeCommand<void>("open_external_url", { sessionToken, url }),
  listConsentTemplates: (sessionToken: string) =>
    invokeCommand<ConsentTemplateSummary[]>("list_consent_templates", { sessionToken }),
  createConsentTemplate: (sessionToken: string, input: Record<string, unknown>) =>
    invokeCommand<ConsentTemplateSummary>("create_consent_template", { sessionToken, input }),
  getReportsSummary: (sessionToken: string, input: { dateFrom: string; dateTo: string }) =>
    invokeCommand<ReportsSummary>("get_reports_summary", { sessionToken, input }),
  saveReportFile: (sessionToken: string, input: Record<string, unknown>) =>
    invokeCommand<ReportExportResult>("save_report_file", { sessionToken, input }),
  updateClinicSettings: (sessionToken: string, input: Record<string, unknown>) =>
    invokeCommand<ClinicSummary>("update_clinic_settings", { sessionToken, input }),
  listMessageTemplates: (sessionToken: string) =>
    invokeCommand<MessageTemplateSummary[]>("list_message_templates", { sessionToken }),
  globalSearch: (sessionToken: string, term: string) =>
    invokeCommand<GlobalSearchResult[]>("global_search", { sessionToken, term }),
  listRoles: (sessionToken: string) => invokeCommand<RoleSummary[]>("list_roles", { sessionToken }),
  createUser: (sessionToken: string, input: Record<string, unknown>) =>
    invokeCommand<UserListItem>("create_user", { sessionToken, input }),
  listPeriodontalRecords: (sessionToken: string) =>
    invokeCommand<PeriodontalRecordSummary[]>("list_periodontal_records", { sessionToken }),
  createPeriodontalRecord: (sessionToken: string, input: Record<string, unknown>) =>
    invokeCommand<PeriodontalRecordSummary>("create_periodontal_record", { sessionToken, input }),
};
