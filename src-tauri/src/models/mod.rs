use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClinicSummary {
    pub id: String,
    pub name: String,
    pub subtitle: String,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapStatus {
    pub requires_setup: bool,
    pub clinic: Option<ClinicSummary>,
    pub license: LicenseStatus,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LicenseStatus {
    pub status: String,
    pub access_mode: String,
    pub can_write: bool,
    pub message: String,
    pub trial_started_at: Option<String>,
    pub trial_ends_at: Option<String>,
    pub activated_at: Option<String>,
    pub last_check_at: Option<String>,
    pub next_check_at: Option<String>,
    pub device_id: Option<String>,
    pub installation_id: Option<String>,
    pub clinic_id: Option<String>,
    pub customer_id: Option<String>,
    pub subscription_id: Option<String>,
    pub plan_code: Option<String>,
    pub plan_limits: Option<Value>,
    pub days_remaining: i64,
    pub is_trial_active: bool,
    pub is_expired: bool,
    pub is_licensed: bool,
    pub requires_activation: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupInput {
    pub clinic_name: String,
    pub clinic_phone: Option<String>,
    pub clinic_whatsapp: Option<String>,
    pub admin_full_name: String,
    pub admin_username: String,
    pub admin_email: Option<String>,
    pub admin_password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, FromRow, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: String,
    pub clinic_id: String,
    pub full_name: String,
    pub username: String,
    pub email: Option<String>,
    pub role_name: Option<String>,
    pub professional_license: Option<String>,
    pub specialty: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthSession {
    pub session_token: String,
    pub expires_at: String,
    pub user: UserProfile,
    pub permissions: Vec<String>,
    pub license: LicenseStatus,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserListItem {
    pub id: String,
    pub full_name: String,
    pub username: String,
    pub role_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePatientInput {
    pub full_name: String,
    pub birth_date: Option<String>,
    pub sex: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub occupation: Option<String>,
    pub allergies: Option<String>,
    pub systemic_diseases: Option<String>,
    pub current_medications: Option<String>,
    pub relevant_history: Option<String>,
    pub habits: Option<String>,
    pub general_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePatientInput {
    pub id: String,
    pub full_name: String,
    pub birth_date: Option<String>,
    pub sex: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub occupation: Option<String>,
    pub allergies: Option<String>,
    pub systemic_diseases: Option<String>,
    pub current_medications: Option<String>,
    pub relevant_history: Option<String>,
    pub habits: Option<String>,
    pub general_notes: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize, FromRow, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PatientSummary {
    pub id: String,
    pub full_name: String,
    pub birth_date: Option<String>,
    pub sex: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub occupation: Option<String>,
    pub allergies: Option<String>,
    pub systemic_diseases: Option<String>,
    pub current_medications: Option<String>,
    pub relevant_history: Option<String>,
    pub habits: Option<String>,
    pub general_notes: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPatientsInput {
    pub search: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAppointmentInput {
    pub patient_id: String,
    pub dentist_user_id: Option<String>,
    pub starts_at: String,
    pub duration_minutes: i64,
    pub reason: String,
    pub appointment_type: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAppointmentInput {
    pub id: String,
    pub patient_id: String,
    pub dentist_user_id: Option<String>,
    pub starts_at: String,
    pub duration_minutes: i64,
    pub reason: String,
    pub appointment_type: String,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAppointmentsInput {
    pub date: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAppointmentStatusInput {
    pub appointment_id: String,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, FromRow, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppointmentSummary {
    pub id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub patient_phone: Option<String>,
    pub patient_whatsapp: Option<String>,
    pub patient_email: Option<String>,
    pub dentist_user_id: Option<String>,
    pub dentist_name: Option<String>,
    pub starts_at: String,
    pub ends_at: String,
    pub duration_minutes: i64,
    pub reason: String,
    pub appointment_type: String,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClinicalRecordInput {
    pub patient_id: String,
    pub appointment_id: Option<String>,
    pub chief_complaint: Option<String>,
    pub current_condition: Option<String>,
    pub hereditary_history: Option<String>,
    pub pathological_history: Option<String>,
    pub non_pathological_history: Option<String>,
    pub allergies: Option<String>,
    pub current_medications: Option<String>,
    pub systemic_diseases: Option<String>,
    pub habits: Option<String>,
    pub clinical_exploration: Option<String>,
    pub diagnosis: Option<String>,
    pub prognosis: Option<String>,
    pub suggested_plan: Option<String>,
    pub indications: Option<String>,
    pub observations: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClinicalEvolutionInput {
    pub patient_id: String,
    pub clinical_record_id: Option<String>,
    pub appointment_id: Option<String>,
    pub reason: String,
    pub findings: Option<String>,
    pub procedures_done: Option<String>,
    pub indications: Option<String>,
    pub next_appointment_notes: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClinicalRecordSummary {
    pub id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub responsible_user_id: Option<String>,
    pub responsible_name: Option<String>,
    pub chief_complaint: Option<String>,
    pub diagnosis: Option<String>,
    pub suggested_plan: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClinicalEvolutionSummary {
    pub id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub responsible_name: String,
    pub reason: String,
    pub findings: Option<String>,
    pub procedures_done: Option<String>,
    pub indications: Option<String>,
    pub next_appointment_notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatientIdInput {
    pub patient_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertOdontogramEntryInput {
    pub patient_id: String,
    pub dentition_type: String,
    pub tooth_number: String,
    pub surface: Option<String>,
    pub state: String,
    pub finding: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct OdontogramEntry {
    pub id: String,
    pub tooth_number: String,
    pub surface: Option<String>,
    pub state: String,
    pub finding: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OdontogramRecordView {
    pub id: String,
    pub patient_id: String,
    pub dentition_type: String,
    pub entries: Vec<OdontogramEntry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    pub appointments_today: i64,
    pub confirmed_today: i64,
    pub unconfirmed_today: i64,
    pub waiting_today: i64,
    pub revenue_today_cents: i64,
    pub revenue_week_cents: i64,
    pub revenue_month_cents: i64,
    pub pending_estimates: i64,
    pub approved_estimates: i64,
    pub new_patients_month: i64,
    pub low_inventory: i64,
    pub open_alerts: i64,
    pub active_treatment_plans: i64,
    pub upcoming_appointments: Vec<AppointmentSummary>,
    pub income_series: Vec<ChartPoint>,
    pub appointment_statuses: Vec<ChartPoint>,
    pub payment_methods: Vec<ChartPoint>,
    pub critical_alerts: Vec<AlertSummary>,
    pub restock_items: Vec<RestockReportItem>,
}

#[derive(Debug, Serialize, FromRow, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartPoint {
    pub label: String,
    pub value: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupResult {
    pub id: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_at: String,
    pub checksum_sha256: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BackupSummary {
    pub id: String,
    pub path: String,
    pub status: String,
    pub backup_type: String,
    pub size_bytes: i64,
    pub checksum_sha256: Option<String>,
    pub verification_status: Option<String>,
    pub verified_at: Option<String>,
    pub app_version: Option<String>,
    pub migration_version: Option<i64>,
    pub file_count: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSettings {
    pub automatic_enabled: bool,
    pub frequency: String,
    pub include_files: bool,
    pub encrypt_backups: bool,
    pub retention_limit: i64,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBackupSettingsInput {
    pub automatic_enabled: bool,
    pub frequency: String,
    pub include_files: bool,
    pub encrypt_backups: bool,
    pub retention_limit: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BackupManifest {
    pub backup_id: String,
    pub clinic_id: Option<String>,
    pub created_at: String,
    pub app_version: String,
    pub database_version: String,
    pub migration_version: i64,
    pub includes: Vec<String>,
    pub table_counts: Value,
    pub file_count: i64,
    pub checksum: Value,
    pub compression: String,
    pub encrypted: bool,
    pub created_by_user_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupVerificationResult {
    pub valid: bool,
    pub path: String,
    pub backup_id: Option<String>,
    pub archive_checksum_sha256: Option<String>,
    pub manifest: Option<BackupManifest>,
    pub checked_files: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePreview {
    pub valid: bool,
    pub source_path: String,
    pub manifest: Option<BackupManifest>,
    pub summary: Value,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePrepareResult {
    pub restore_job_id: String,
    pub staged_path: String,
    pub safety_backup_path: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTreatmentInput {
    pub name: String,
    pub category: String,
    pub description: Option<String>,
    pub base_price_cents: i64,
    pub estimated_duration_minutes: Option<i64>,
    pub requires_follow_up: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTreatmentInput {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: Option<String>,
    pub base_price_cents: i64,
    pub estimated_duration_minutes: Option<i64>,
    pub requires_follow_up: bool,
    pub active: bool,
}

#[derive(Debug, Serialize, FromRow, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TreatmentCatalogItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: Option<String>,
    pub base_price_cents: i64,
    pub estimated_duration_minutes: Option<i64>,
    pub requires_follow_up: bool,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreatmentPlanItemInput {
    pub treatment_catalog_id: Option<String>,
    pub tooth_number: Option<String>,
    pub diagnosis: Option<String>,
    pub phase: Option<String>,
    pub priority: Option<String>,
    pub quantity: i64,
    pub unit_price_cents: i64,
    pub discount_cents: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTreatmentPlanInput {
    pub patient_id: String,
    pub diagnosis: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<TreatmentPlanItemInput>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TreatmentPlanSummary {
    pub id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub diagnosis: Option<String>,
    pub subtotal_cents: i64,
    pub discount_cents: i64,
    pub total_cents: i64,
    pub paid_cents: i64,
    pub balance_cents: i64,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TreatmentPlanItemView {
    pub id: String,
    pub treatment_plan_id: String,
    pub treatment_name: Option<String>,
    pub tooth_number: Option<String>,
    pub diagnosis: Option<String>,
    pub phase: Option<String>,
    pub priority: Option<String>,
    pub quantity: i64,
    pub unit_price_cents: i64,
    pub discount_cents: i64,
    pub total_cents: i64,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EstimateItemInput {
    pub treatment_catalog_id: Option<String>,
    pub description: String,
    pub quantity: i64,
    pub unit_price_cents: i64,
    pub discount_cents: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEstimateInput {
    pub patient_id: String,
    pub treatment_plan_id: Option<String>,
    pub valid_until: Option<String>,
    pub observations: Option<String>,
    pub terms: Option<String>,
    pub items: Vec<EstimateItemInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusInput {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct EstimateSummary {
    pub id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub treatment_plan_id: Option<String>,
    pub folio: String,
    pub status: String,
    pub valid_until: Option<String>,
    pub subtotal_cents: i64,
    pub discount_cents: i64,
    pub total_cents: i64,
    pub observations: Option<String>,
    pub terms: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct EstimateItemView {
    pub id: String,
    pub estimate_id: String,
    pub description: String,
    pub quantity: i64,
    pub unit_price_cents: i64,
    pub discount_cents: i64,
    pub total_cents: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterPaymentInput {
    pub patient_id: String,
    pub concept: String,
    pub amount_cents: i64,
    pub method: String,
    pub notes: Option<String>,
    pub estimate_id: Option<String>,
    pub treatment_plan_item_id: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PaymentSummary {
    pub id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub folio: String,
    pub concept: String,
    pub amount_cents: i64,
    pub method: String,
    pub status: String,
    pub paid_at: String,
    pub received_by_name: String,
    pub notes: Option<String>,
    pub proof_files_count: i64,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CashRegisterSummary {
    pub id: String,
    pub opened_by_name: String,
    pub opened_at: String,
    pub opening_float_cents: i64,
    pub status: String,
    pub closed_at: Option<String>,
    pub total_cash_cents: i64,
    pub total_transfer_cents: i64,
    pub total_card_cents: i64,
    pub total_other_cents: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCashRegisterInput {
    pub opening_float_cents: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseCashRegisterInput {
    pub cash_register_id: String,
    pub counted_cash_cents: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CashClosureResult {
    pub cash_register_id: String,
    pub expected_cash_cents: i64,
    pub counted_cash_cents: i64,
    pub difference_cents: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSupplierInput {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SupplierSummary {
    pub id: String,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub notes: Option<String>,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInventoryItemInput {
    pub supplier_id: Option<String>,
    pub name: String,
    pub category: String,
    pub unit: String,
    pub current_quantity: f64,
    pub minimum_stock: f64,
    pub cost_cents: i64,
    pub purchase_date: Option<String>,
    pub expiration_date: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInventoryItemInput {
    pub id: String,
    pub supplier_id: Option<String>,
    pub name: String,
    pub category: String,
    pub unit: String,
    pub current_quantity: f64,
    pub minimum_stock: f64,
    pub cost_cents: i64,
    pub purchase_date: Option<String>,
    pub expiration_date: Option<String>,
    pub location: Option<String>,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInventoryMovementInput {
    pub inventory_item_id: String,
    pub movement_type: String,
    pub quantity: f64,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItemSummary {
    pub id: String,
    pub supplier_id: Option<String>,
    pub supplier_name: Option<String>,
    pub name: String,
    pub category: String,
    pub unit: String,
    pub current_quantity: f64,
    pub minimum_stock: f64,
    pub cost_cents: i64,
    pub expiration_date: Option<String>,
    pub location: Option<String>,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAlertInput {
    pub patient_id: Option<String>,
    pub alert_type: String,
    pub priority: String,
    pub title: String,
    pub message: String,
    pub due_at: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AlertSummary {
    pub id: String,
    pub patient_name: Option<String>,
    pub alert_type: String,
    pub priority: String,
    pub title: String,
    pub message: String,
    pub due_at: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePatientFileInput {
    pub patient_id: String,
    pub category_name: String,
    pub original_name: String,
    pub mime_type: Option<String>,
    pub description: Option<String>,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PatientFileSummary {
    pub id: String,
    pub patient_id: Option<String>,
    pub patient_name: Option<String>,
    pub category_name: Option<String>,
    pub file_type: String,
    pub original_name: String,
    pub relative_path: String,
    pub mime_type: Option<String>,
    pub size_bytes: i64,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConsentTemplateInput {
    pub name: String,
    pub treatment_category: Option<String>,
    pub body: String,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ConsentTemplateSummary {
    pub id: String,
    pub name: String,
    pub treatment_category: Option<String>,
    pub body: String,
    pub active: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportsSummary {
    pub income_cents: i64,
    pub payments_count: i64,
    pub appointments_count: i64,
    pub cancelled_appointments: i64,
    pub new_patients: i64,
    pub estimates_total: i64,
    pub estimates_approved: i64,
    pub pending_balances_cents: i64,
    pub low_inventory: i64,
    pub restock_items: Vec<RestockReportItem>,
    pub income_by_method: Vec<ChartPoint>,
    pub appointments_by_status: Vec<ChartPoint>,
}

#[derive(Debug, Serialize, FromRow, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RestockReportItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub unit: String,
    pub current_quantity: f64,
    pub minimum_stock: f64,
    pub suggested_quantity: f64,
    pub cost_cents: i64,
    pub estimated_cost_cents: i64,
    pub supplier_name: Option<String>,
    pub expiration_date: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportsFilterInput {
    pub date_from: String,
    pub date_to: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveReportFileInput {
    pub report_type: String,
    pub format: String,
    pub file_name: String,
    pub target_path: Option<String>,
    pub filters_json: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportExportResult {
    pub id: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_at: String,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSearchResult {
    pub entity_type: String,
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub route: String,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClinicSettingsInput {
    pub name: String,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub tax_data: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MessageTemplateSummary {
    pub id: String,
    pub name: String,
    pub body: String,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RoleSummary {
    pub id: String,
    pub name: String,
    pub system_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserInput {
    pub full_name: String,
    pub username: String,
    pub email: Option<String>,
    pub role_id: String,
    pub password: String,
    pub professional_license: Option<String>,
    pub specialty: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePeriodontalRecordInput {
    pub patient_id: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PeriodontalRecordSummary {
    pub id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterInstallationInput {
    pub portal_base_url: Option<String>,
    pub pairing_code: Option<String>,
    pub device_label: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevokeLocalDeviceInput {
    pub device_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, FromRow, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncDeviceSummary {
    pub id: String,
    pub installation_id: String,
    pub device_id: String,
    pub device_label: Option<String>,
    pub portal_base_url: Option<String>,
    pub status: String,
    pub last_registered_at: Option<String>,
    pub last_refreshed_at: Option<String>,
    pub last_sync_at: Option<String>,
    pub last_error: Option<String>,
    pub revoked_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub configured: bool,
    pub active_device: Option<SyncDeviceSummary>,
    pub devices: Vec<SyncDeviceSummary>,
    pub pending_outbox: i64,
    pub failed_outbox: i64,
    pub pending_inbox: i64,
    pub pending_receipts: i64,
    pub pull_cursor: Option<String>,
    pub last_sync_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterInstallationResult {
    pub status: SyncStatus,
    pub installation_id: String,
    pub device_id: String,
    pub paired: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncRunResult {
    pub pushed_events: i64,
    pub applied_commands: i64,
    pub failed_commands: i64,
    pub acked_receipts: i64,
    pub status: SyncStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoteCommandRequestedBy {
    pub user_id: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoteAppointmentStatusCommand {
    pub command_type: String,
    pub command_id: String,
    pub clinic_id: Option<String>,
    pub appointment_id: String,
    pub payload: Value,
    pub requested_by: Option<RemoteCommandRequestedBy>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoteCommandApplyResult {
    pub command_id: String,
    pub appointment_id: String,
    pub status: String,
    pub applied_at: Option<String>,
    pub message: Option<String>,
}
