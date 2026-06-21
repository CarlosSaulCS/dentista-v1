use tauri::State;

use crate::database::AppState;
use crate::errors::command_error;
use crate::models::{
    RegisterInstallationInput, RegisterInstallationResult, RevokeLocalDeviceInput, SyncRunResult,
    SyncStatus,
};
use crate::services::sync_service;

#[tauri::command]
pub async fn register_installation(
    state: State<'_, AppState>,
    session_token: String,
    input: RegisterInstallationInput,
) -> Result<RegisterInstallationResult, String> {
    sync_service::register_installation(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn refresh_sync_token(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<SyncStatus, String> {
    sync_service::refresh_sync_token(&state.db, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn sync_now(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<SyncRunResult, String> {
    sync_service::sync_now(&state.db, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn get_sync_status(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<SyncStatus, String> {
    sync_service::get_sync_status(&state.db, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn revoke_local_device(
    state: State<'_, AppState>,
    session_token: String,
    input: RevokeLocalDeviceInput,
) -> Result<SyncStatus, String> {
    sync_service::revoke_local_device(&state.db, &session_token, input)
        .await
        .map_err(command_error)
}
