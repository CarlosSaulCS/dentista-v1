use tauri::State;

use crate::database::AppState;
use crate::errors::command_error;
use crate::models::{OdontogramRecordView, UpsertOdontogramEntryInput};
use crate::services::odontogram_service;

#[tauri::command]
pub async fn get_odontogram(
    state: State<'_, AppState>,
    session_token: String,
    patient_id: String,
    dentition_type: String,
) -> Result<OdontogramRecordView, String> {
    odontogram_service::get_odontogram(&state.db, &session_token, &patient_id, &dentition_type)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn upsert_odontogram_entry(
    state: State<'_, AppState>,
    session_token: String,
    input: UpsertOdontogramEntryInput,
) -> Result<OdontogramRecordView, String> {
    odontogram_service::upsert_odontogram_entry(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}
