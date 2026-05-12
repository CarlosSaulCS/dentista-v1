use tauri::State;

use crate::database::AppState;
use crate::errors::command_error;
use crate::models::{CreatePatientInput, ListPatientsInput, PatientSummary};
use crate::services::patient_service;

#[tauri::command]
pub async fn create_patient(
    state: State<'_, AppState>,
    session_token: String,
    input: CreatePatientInput,
) -> Result<PatientSummary, String> {
    patient_service::create_patient(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_patients(
    state: State<'_, AppState>,
    session_token: String,
    input: ListPatientsInput,
) -> Result<Vec<PatientSummary>, String> {
    patient_service::list_patients(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn get_patient(
    state: State<'_, AppState>,
    session_token: String,
    patient_id: String,
) -> Result<PatientSummary, String> {
    patient_service::get_patient(&state.db, &session_token, &patient_id)
        .await
        .map_err(command_error)
}
