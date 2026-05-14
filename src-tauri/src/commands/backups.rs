use tauri::State;

use crate::database::AppState;
use crate::errors::command_error;
use crate::models::{
    BackupResult, BackupSettings, BackupSummary, BackupVerificationResult, RestorePrepareResult,
    RestorePreview, UpdateBackupSettingsInput,
};
use crate::services::backup_service;

#[tauri::command]
pub async fn create_backup(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<BackupResult, String> {
    backup_service::create_backup(&state, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_backups(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<Vec<BackupSummary>, String> {
    backup_service::list_backups(&state, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn verify_backup(
    state: State<'_, AppState>,
    session_token: String,
    path: String,
) -> Result<BackupVerificationResult, String> {
    backup_service::verify_backup(&state, &session_token, &path)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn preview_restore(
    state: State<'_, AppState>,
    session_token: String,
    path: String,
) -> Result<RestorePreview, String> {
    backup_service::preview_restore(&state, &session_token, &path)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn prepare_restore(
    state: State<'_, AppState>,
    session_token: String,
    path: String,
) -> Result<RestorePrepareResult, String> {
    backup_service::prepare_restore(&state, &session_token, &path)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn get_backup_settings(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<BackupSettings, String> {
    backup_service::get_backup_settings(&state, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn update_backup_settings(
    state: State<'_, AppState>,
    session_token: String,
    input: UpdateBackupSettingsInput,
) -> Result<BackupSettings, String> {
    backup_service::update_backup_settings(&state, &session_token, input)
        .await
        .map_err(command_error)
}
