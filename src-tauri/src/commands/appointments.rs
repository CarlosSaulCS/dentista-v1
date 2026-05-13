use tauri::State;

use crate::database::AppState;
use crate::errors::command_error;
use crate::models::{
    AppointmentSummary, CreateAppointmentInput, ListAppointmentsInput, UpdateAppointmentInput,
    UpdateAppointmentStatusInput,
};
use crate::services::appointment_service;

#[tauri::command]
pub async fn create_appointment(
    state: State<'_, AppState>,
    session_token: String,
    input: CreateAppointmentInput,
) -> Result<AppointmentSummary, String> {
    appointment_service::create_appointment(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_appointments(
    state: State<'_, AppState>,
    session_token: String,
    input: ListAppointmentsInput,
) -> Result<Vec<AppointmentSummary>, String> {
    appointment_service::list_appointments(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn get_next_patient_appointment(
    state: State<'_, AppState>,
    session_token: String,
    patient_id: String,
) -> Result<Option<AppointmentSummary>, String> {
    appointment_service::get_next_patient_appointment(&state.db, &session_token, &patient_id)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn update_appointment(
    state: State<'_, AppState>,
    session_token: String,
    input: UpdateAppointmentInput,
) -> Result<AppointmentSummary, String> {
    appointment_service::update_appointment(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn update_appointment_status(
    state: State<'_, AppState>,
    session_token: String,
    input: UpdateAppointmentStatusInput,
) -> Result<AppointmentSummary, String> {
    appointment_service::update_appointment_status(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn soft_delete_appointment(
    state: State<'_, AppState>,
    session_token: String,
    appointment_id: String,
) -> Result<AppointmentSummary, String> {
    appointment_service::soft_delete_appointment(&state.db, &session_token, &appointment_id)
        .await
        .map_err(command_error)
}
