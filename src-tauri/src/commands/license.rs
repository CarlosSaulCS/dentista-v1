use tauri::State;

use crate::database::AppState;
use crate::errors::command_error;
use crate::models::LicenseStatus;
use crate::services::license_service;

#[tauri::command]
pub async fn get_license_status(state: State<'_, AppState>) -> Result<LicenseStatus, String> {
    license_service::get_license_status(&state.db)
        .await
        .map_err(command_error)
}
