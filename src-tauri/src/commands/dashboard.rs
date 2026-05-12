use tauri::State;

use crate::database::AppState;
use crate::errors::command_error;
use crate::models::DashboardSummary;
use crate::services::dashboard_service;

#[tauri::command]
pub async fn get_dashboard_summary(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<DashboardSummary, String> {
    dashboard_service::get_dashboard_summary(&state.db, &session_token)
        .await
        .map_err(command_error)
}
