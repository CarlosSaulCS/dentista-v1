use tauri::State;

use crate::database::AppState;
use crate::errors::command_error;
use crate::models::*;
use crate::services::office_service;

#[tauri::command]
pub async fn list_treatments(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<Vec<TreatmentCatalogItem>, String> {
    office_service::list_treatments(&state.db, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn create_treatment(
    state: State<'_, AppState>,
    session_token: String,
    input: CreateTreatmentInput,
) -> Result<TreatmentCatalogItem, String> {
    office_service::create_treatment(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn update_treatment(
    state: State<'_, AppState>,
    session_token: String,
    input: UpdateTreatmentInput,
) -> Result<TreatmentCatalogItem, String> {
    office_service::update_treatment(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_treatment_plans(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<Vec<TreatmentPlanSummary>, String> {
    office_service::list_treatment_plans(&state.db, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn create_treatment_plan(
    state: State<'_, AppState>,
    session_token: String,
    input: CreateTreatmentPlanInput,
) -> Result<TreatmentPlanSummary, String> {
    office_service::create_treatment_plan(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_treatment_plan_items(
    state: State<'_, AppState>,
    session_token: String,
    treatment_plan_id: String,
) -> Result<Vec<TreatmentPlanItemView>, String> {
    office_service::list_treatment_plan_items(&state.db, &session_token, &treatment_plan_id)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_estimates(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<Vec<EstimateSummary>, String> {
    office_service::list_estimates(&state.db, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn create_estimate(
    state: State<'_, AppState>,
    session_token: String,
    input: CreateEstimateInput,
) -> Result<EstimateSummary, String> {
    office_service::create_estimate(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn update_estimate_status(
    state: State<'_, AppState>,
    session_token: String,
    input: UpdateStatusInput,
) -> Result<EstimateSummary, String> {
    office_service::update_estimate_status(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_estimate_items(
    state: State<'_, AppState>,
    session_token: String,
    estimate_id: String,
) -> Result<Vec<EstimateItemView>, String> {
    office_service::list_estimate_items(&state.db, &session_token, &estimate_id)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_payments(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<Vec<PaymentSummary>, String> {
    office_service::list_payments(&state.db, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn register_payment(
    state: State<'_, AppState>,
    session_token: String,
    input: RegisterPaymentInput,
) -> Result<PaymentSummary, String> {
    office_service::register_payment(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn get_current_cash_register(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<Option<CashRegisterSummary>, String> {
    office_service::get_current_cash_register(&state.db, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn open_cash_register(
    state: State<'_, AppState>,
    session_token: String,
    input: OpenCashRegisterInput,
) -> Result<CashRegisterSummary, String> {
    office_service::open_cash_register(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn close_cash_register(
    state: State<'_, AppState>,
    session_token: String,
    input: CloseCashRegisterInput,
) -> Result<CashClosureResult, String> {
    office_service::close_cash_register(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_suppliers(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<Vec<SupplierSummary>, String> {
    office_service::list_suppliers(&state.db, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn create_supplier(
    state: State<'_, AppState>,
    session_token: String,
    input: CreateSupplierInput,
) -> Result<SupplierSummary, String> {
    office_service::create_supplier(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_inventory_items(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<Vec<InventoryItemSummary>, String> {
    office_service::list_inventory_items(&state.db, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn create_inventory_item(
    state: State<'_, AppState>,
    session_token: String,
    input: CreateInventoryItemInput,
) -> Result<InventoryItemSummary, String> {
    office_service::create_inventory_item(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn update_inventory_item(
    state: State<'_, AppState>,
    session_token: String,
    input: UpdateInventoryItemInput,
) -> Result<InventoryItemSummary, String> {
    office_service::update_inventory_item(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn soft_delete_inventory_item(
    state: State<'_, AppState>,
    session_token: String,
    inventory_item_id: String,
) -> Result<InventoryItemSummary, String> {
    office_service::soft_delete_inventory_item(&state.db, &session_token, &inventory_item_id)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn create_inventory_movement(
    state: State<'_, AppState>,
    session_token: String,
    input: CreateInventoryMovementInput,
) -> Result<InventoryItemSummary, String> {
    office_service::create_inventory_movement(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_alerts(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<Vec<AlertSummary>, String> {
    office_service::list_alerts(&state.db, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn create_alert(
    state: State<'_, AppState>,
    session_token: String,
    input: CreateAlertInput,
) -> Result<AlertSummary, String> {
    office_service::create_alert(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn resolve_alert(
    state: State<'_, AppState>,
    session_token: String,
    id: String,
) -> Result<AlertSummary, String> {
    office_service::resolve_alert(&state.db, &session_token, &id)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn save_patient_file(
    state: State<'_, AppState>,
    session_token: String,
    input: SavePatientFileInput,
) -> Result<PatientFileSummary, String> {
    office_service::save_patient_file(&state, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_patient_files(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<Vec<PatientFileSummary>, String> {
    office_service::list_patient_files(&state.db, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn open_patient_file(
    state: State<'_, AppState>,
    session_token: String,
    file_id: String,
) -> Result<(), String> {
    office_service::open_patient_file(&state, &session_token, &file_id)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn open_external_url(
    state: State<'_, AppState>,
    session_token: String,
    url: String,
) -> Result<(), String> {
    office_service::open_external_url(&state.db, &session_token, &url)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_consent_templates(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<Vec<ConsentTemplateSummary>, String> {
    office_service::list_consent_templates(&state.db, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn create_consent_template(
    state: State<'_, AppState>,
    session_token: String,
    input: CreateConsentTemplateInput,
) -> Result<ConsentTemplateSummary, String> {
    office_service::create_consent_template(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn get_reports_summary(
    state: State<'_, AppState>,
    session_token: String,
    input: ReportsFilterInput,
) -> Result<ReportsSummary, String> {
    office_service::get_reports_summary(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn update_clinic_settings(
    state: State<'_, AppState>,
    session_token: String,
    input: UpdateClinicSettingsInput,
) -> Result<ClinicSummary, String> {
    office_service::update_clinic_settings(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_message_templates(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<Vec<MessageTemplateSummary>, String> {
    office_service::list_message_templates(&state.db, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn global_search(
    state: State<'_, AppState>,
    session_token: String,
    term: String,
) -> Result<Vec<GlobalSearchResult>, String> {
    office_service::global_search(&state.db, &session_token, &term)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_roles(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<Vec<RoleSummary>, String> {
    office_service::list_roles(&state.db, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn create_user(
    state: State<'_, AppState>,
    session_token: String,
    input: CreateUserInput,
) -> Result<UserListItem, String> {
    office_service::create_user(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_periodontal_records(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<Vec<PeriodontalRecordSummary>, String> {
    office_service::list_periodontal_records(&state.db, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn create_periodontal_record(
    state: State<'_, AppState>,
    session_token: String,
    input: CreatePeriodontalRecordInput,
) -> Result<PeriodontalRecordSummary, String> {
    office_service::create_periodontal_record(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}
