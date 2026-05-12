use tauri::State;

use crate::database::AppState;
use crate::errors::command_error;
use crate::models::{
    ClinicalEvolutionSummary, ClinicalRecordSummary, CreateClinicalEvolutionInput,
    CreateClinicalRecordInput,
};
use crate::services::clinical_service;

#[tauri::command]
pub async fn create_clinical_record(
    state: State<'_, AppState>,
    session_token: String,
    input: CreateClinicalRecordInput,
) -> Result<ClinicalRecordSummary, String> {
    clinical_service::create_clinical_record(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn create_clinical_evolution(
    state: State<'_, AppState>,
    session_token: String,
    input: CreateClinicalEvolutionInput,
) -> Result<ClinicalEvolutionSummary, String> {
    clinical_service::create_clinical_evolution(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_clinical_records(
    state: State<'_, AppState>,
    session_token: String,
    patient_id: String,
) -> Result<Vec<ClinicalRecordSummary>, String> {
    clinical_service::list_clinical_records(&state.db, &session_token, &patient_id)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_clinical_evolutions(
    state: State<'_, AppState>,
    session_token: String,
    patient_id: String,
) -> Result<Vec<ClinicalEvolutionSummary>, String> {
    clinical_service::list_clinical_evolutions(&state.db, &session_token, &patient_id)
        .await
        .map_err(command_error)
}
