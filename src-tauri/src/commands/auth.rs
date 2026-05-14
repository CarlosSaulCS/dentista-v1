use tauri::State;

use crate::database::AppState;
use crate::errors::command_error;
use crate::models::{AuthSession, BootstrapStatus, LoginInput, SetupInput, UserListItem};
use crate::services::{auth_service, backup_service};

#[tauri::command]
pub async fn get_bootstrap_status(state: State<'_, AppState>) -> Result<BootstrapStatus, String> {
    auth_service::get_bootstrap_status(&state.db)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn setup_clinic_and_admin(
    state: State<'_, AppState>,
    input: SetupInput,
) -> Result<AuthSession, String> {
    auth_service::setup_clinic_and_admin(&state.db, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn login(state: State<'_, AppState>, input: LoginInput) -> Result<AuthSession, String> {
    let session = auth_service::login(&state.db, input)
        .await
        .map_err(command_error)?;
    let _ = backup_service::create_automatic_backup_if_due(&state, &session.session_token).await;
    Ok(session)
}

#[tauri::command]
pub async fn logout(state: State<'_, AppState>, session_token: String) -> Result<(), String> {
    auth_service::logout(&state.db, &session_token)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn list_users(
    state: State<'_, AppState>,
    session_token: String,
) -> Result<Vec<UserListItem>, String> {
    auth_service::list_users(&state.db, &session_token)
        .await
        .map_err(command_error)
}
