use tauri::State;

use crate::database::AppState;
use crate::errors::command_error;
use crate::models::{ReportExportResult, SaveReportFileInput};
use crate::reports;

#[tauri::command]
pub async fn save_report_file(
    state: State<'_, AppState>,
    session_token: String,
    input: SaveReportFileInput,
) -> Result<ReportExportResult, String> {
    reports::save_report_file(&state, &session_token, input)
        .await
        .map_err(command_error)
}
