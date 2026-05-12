use tauri::State;

use crate::database::AppState;
use crate::errors::command_error;
use crate::models::BackupResult;
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
