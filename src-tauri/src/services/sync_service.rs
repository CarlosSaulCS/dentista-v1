use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{FromRow, SqlitePool};

use crate::errors::{AppError, AppResult};
use crate::models::{
    RegisterInstallationInput, RegisterInstallationResult, RemoteAppointmentStatusCommand,
    RevokeLocalDeviceInput, SyncDeviceSummary, SyncRunResult, SyncStatus,
};
use crate::services::appointment_service;
use crate::services::audit_service::log_action;
use crate::services::auth_service::{validate_session, validate_session_for_intent};
use crate::services::license_service::{self, AccessIntent};
use crate::utils::{new_id, now_utc};

const SYNC_API_PREFIX: &str = "/api/dental/local-sync";
const MAX_OUTBOX_ATTEMPTS: i64 = 5;

#[derive(Debug, FromRow, Clone)]
struct SyncDeviceRecord {
    id: String,
    clinic_id: String,
    installation_id: String,
    device_id: String,
    portal_base_url: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
}

#[derive(Debug, FromRow)]
struct OutboxRow {
    id: String,
    aggregate_type: String,
    aggregate_id: String,
    event_type: String,
    payload_json: String,
    occurred_at: String,
}

#[derive(Debug, FromRow)]
struct ReceiptRow {
    command_id: String,
    status: String,
    result_json: Option<String>,
    error_message: Option<String>,
    applied_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RegisterInstallationRequest<'a> {
    pairing_code: &'a str,
    installation_id: &'a str,
    device_id: &'a str,
    device_label: Option<&'a str>,
    app_version: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterInstallationResponse {
    access_token: String,
    refresh_token: String,
    token_expires_at: String,
    refresh_token_expires_at: String,
    status: Option<String>,
    installation_id: Option<String>,
    device_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RefreshTokenRequest<'a> {
    installation_id: &'a str,
    device_id: &'a str,
    refresh_token: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RefreshTokenResponse {
    access_token: String,
    refresh_token: String,
    token_expires_at: String,
    refresh_token_expires_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PushEvent {
    local_event_id: String,
    aggregate_type: String,
    aggregate_id: String,
    event_type: String,
    payload: Value,
    occurred_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PushRequest {
    events: Vec<PushEvent>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PushResponse {
    #[serde(default)]
    accepted_event_ids: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PullResponse {
    #[serde(default)]
    commands: Vec<RemoteAppointmentStatusCommand>,
    cursor: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AckReceipt {
    command_id: String,
    status: String,
    result: Option<Value>,
    error_message: Option<String>,
    applied_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AckRequest {
    receipts: Vec<AckReceipt>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct AckResponse {
    #[serde(default)]
    acked_command_ids: Vec<String>,
}

pub async fn register_installation(
    db: &SqlitePool,
    session_token: &str,
    input: RegisterInstallationInput,
) -> AppResult<RegisterInstallationResult> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("sync.manage"),
        AccessIntent::DataWrite,
    )
    .await?;
    let portal_base_url = normalize_portal_base_url(&input.portal_base_url)?;
    let (installation_id, device_id) = ensure_local_device_ids(db, &ctx.clinic_id).await?;
    let label = input
        .device_label
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Dentista v1 Professional");
    let pairing_code = input
        .pairing_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let now = now_utc();

    let mut access_token = None;
    let mut refresh_token = None;
    let mut token_expires_at = None;
    let mut refresh_token_expires_at = None;
    let mut status = "pending_pairing".to_string();
    let mut paired = false;
    let mut final_installation_id = installation_id.clone();
    let mut final_device_id = device_id.clone();

    if let Some(code) = pairing_code {
        let response = register_with_portal(
            &portal_base_url,
            RegisterInstallationRequest {
                pairing_code: code,
                installation_id: &installation_id,
                device_id: &device_id,
                device_label: Some(label),
                app_version: env!("CARGO_PKG_VERSION"),
            },
        )
        .await?;
        final_installation_id = response.installation_id.unwrap_or(installation_id);
        final_device_id = response.device_id.unwrap_or(device_id);
        access_token = Some(response.access_token);
        refresh_token = Some(response.refresh_token);
        token_expires_at = Some(response.token_expires_at);
        refresh_token_expires_at = Some(response.refresh_token_expires_at);
        status = response.status.unwrap_or_else(|| "active".to_string());
        paired = status == "active";
    }

    sqlx::query(
        r#"
        INSERT INTO sync_device_tokens
          (id, clinic_id, installation_id, device_id, device_label, portal_base_url,
           access_token, access_token_expires_at, refresh_token, refresh_token_expires_at,
           status, last_registered_at, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(clinic_id, installation_id, device_id) DO UPDATE SET
          device_label = excluded.device_label,
          portal_base_url = excluded.portal_base_url,
          access_token = COALESCE(excluded.access_token, sync_device_tokens.access_token),
          access_token_expires_at = COALESCE(excluded.access_token_expires_at, sync_device_tokens.access_token_expires_at),
          refresh_token = COALESCE(excluded.refresh_token, sync_device_tokens.refresh_token),
          refresh_token_expires_at = COALESCE(excluded.refresh_token_expires_at, sync_device_tokens.refresh_token_expires_at),
          status = excluded.status,
          last_registered_at = excluded.last_registered_at,
          last_error = NULL,
          revoked_at = NULL,
          revoked_reason = NULL,
          updated_at = excluded.updated_at
        "#,
    )
    .bind(new_id())
    .bind(&ctx.clinic_id)
    .bind(&final_installation_id)
    .bind(&final_device_id)
    .bind(label)
    .bind(&portal_base_url)
    .bind(access_token)
    .bind(token_expires_at)
    .bind(refresh_token)
    .bind(refresh_token_expires_at)
    .bind(&status)
    .bind(&now)
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;

    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "sync.installation_registered",
        "sync_device_tokens",
        Some(&final_device_id),
        "security",
        Some(json!({
            "paired": paired,
            "portalBaseUrl": portal_base_url,
            "installationId": final_installation_id
        })),
    )
    .await?;

    Ok(RegisterInstallationResult {
        status: sync_status_for_clinic(db, &ctx.clinic_id).await?,
        installation_id: final_installation_id,
        device_id: final_device_id,
        paired,
    })
}

pub async fn refresh_sync_token(db: &SqlitePool, session_token: &str) -> AppResult<SyncStatus> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("sync.manage"),
        AccessIntent::DataWrite,
    )
    .await?;
    let device = active_device(db, &ctx.clinic_id).await?;
    refresh_device_token(db, &device).await?;
    Ok(sync_status_for_clinic(db, &ctx.clinic_id).await?)
}

pub async fn get_sync_status(db: &SqlitePool, session_token: &str) -> AppResult<SyncStatus> {
    let ctx = validate_session(db, session_token, Some("sync.view")).await?;
    sync_status_for_clinic(db, &ctx.clinic_id).await
}

pub async fn revoke_local_device(
    db: &SqlitePool,
    session_token: &str,
    input: RevokeLocalDeviceInput,
) -> AppResult<SyncStatus> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("sync.manage"),
        AccessIntent::DataWrite,
    )
    .await?;
    let now = now_utc();
    sqlx::query(
        r#"
        UPDATE sync_device_tokens
        SET status = 'revoked',
            access_token = NULL,
            refresh_token = NULL,
            revoked_at = ?,
            revoked_reason = ?,
            updated_at = ?
        WHERE clinic_id = ? AND device_id = ? AND status <> 'revoked'
        "#,
    )
    .bind(&now)
    .bind(input.reason.as_deref().map(str::trim))
    .bind(&now)
    .bind(&ctx.clinic_id)
    .bind(input.device_id.trim())
    .execute(db)
    .await?;

    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "sync.device_revoked",
        "sync_device_tokens",
        Some(input.device_id.trim()),
        "security",
        Some(json!({ "reason": input.reason })),
    )
    .await?;

    sync_status_for_clinic(db, &ctx.clinic_id).await
}

pub async fn sync_now(db: &SqlitePool, session_token: &str) -> AppResult<SyncRunResult> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("sync.manage"),
        AccessIntent::DataWrite,
    )
    .await?;
    let mut device = active_device(db, &ctx.clinic_id).await?;
    let pushed_events = match push_outbox(db, &device).await {
        Ok(count) => count,
        Err(error) => {
            if is_auth_error(&error) {
                device = refresh_device_token(db, &device).await?;
                push_outbox(db, &device).await?
            } else {
                remember_device_error(db, &device.id, &error.to_string()).await?;
                return Err(error);
            }
        }
    };

    let (applied_commands, failed_commands) = match pull_and_apply_commands(db, &device).await {
        Ok(result) => result,
        Err(error) => {
            if is_auth_error(&error) {
                device = refresh_device_token(db, &device).await?;
                pull_and_apply_commands(db, &device).await?
            } else {
                remember_device_error(db, &device.id, &error.to_string()).await?;
                return Err(error);
            }
        }
    };

    let acked_receipts = match ack_remote_receipts(db, &device).await {
        Ok(count) => count,
        Err(error) => {
            if is_auth_error(&error) {
                device = refresh_device_token(db, &device).await?;
                ack_remote_receipts(db, &device).await?
            } else {
                remember_device_error(db, &device.id, &error.to_string()).await?;
                return Err(error);
            }
        }
    };

    let now = now_utc();
    sqlx::query(
        "UPDATE sync_device_tokens SET last_sync_at = ?, last_error = NULL, updated_at = ? WHERE id = ?",
    )
    .bind(&now)
    .bind(&now)
    .bind(&device.id)
    .execute(db)
    .await?;

    Ok(SyncRunResult {
        pushed_events,
        applied_commands,
        failed_commands,
        acked_receipts,
        status: sync_status_for_clinic(db, &ctx.clinic_id).await?,
    })
}

pub async fn enqueue_local_event(
    db: &SqlitePool,
    clinic_id: &str,
    aggregate_type: &str,
    aggregate_id: &str,
    event_type: &str,
    payload: Value,
) -> AppResult<()> {
    let now = now_utc();
    sqlx::query(
        r#"
        INSERT INTO sync_outbox
          (id, clinic_id, aggregate_type, aggregate_id, event_type, payload_json,
           occurred_at, available_at, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(new_id())
    .bind(clinic_id)
    .bind(aggregate_type)
    .bind(aggregate_id)
    .bind(event_type)
    .bind(payload.to_string())
    .bind(&now)
    .bind(&now)
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn record_failed_remote_command(
    db: &SqlitePool,
    clinic_id: &str,
    command: &RemoteAppointmentStatusCommand,
    error_message: &str,
) -> AppResult<()> {
    let now = now_utc();
    sqlx::query(
        r#"
        INSERT INTO sync_inbox
          (id, clinic_id, command_id, command_type, aggregate_type, aggregate_id,
           payload_json, requested_by_json, received_at, status, failed_at, error_message,
           created_at, updated_at)
        VALUES (?, ?, ?, ?, 'appointments', ?, ?, ?, ?, 'failed', ?, ?, ?, ?)
        ON CONFLICT(clinic_id, command_id) DO UPDATE SET
          status = 'failed',
          failed_at = excluded.failed_at,
          error_message = excluded.error_message,
          updated_at = excluded.updated_at
        "#,
    )
    .bind(new_id())
    .bind(clinic_id)
    .bind(&command.command_id)
    .bind(&command.command_type)
    .bind(&command.appointment_id)
    .bind(command.payload.to_string())
    .bind(
        command
            .requested_by
            .as_ref()
            .map(|requested_by| json!(requested_by).to_string()),
    )
    .bind(&now)
    .bind(&now)
    .bind(error_message)
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO remote_command_receipts
          (id, clinic_id, command_id, command_type, appointment_id, status, error_message,
           received_at, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, 'failed', ?, ?, ?, ?)
        ON CONFLICT(clinic_id, command_id) DO UPDATE SET
          status = 'failed',
          error_message = excluded.error_message,
          updated_at = excluded.updated_at
        "#,
    )
    .bind(new_id())
    .bind(clinic_id)
    .bind(&command.command_id)
    .bind(&command.command_type)
    .bind(&command.appointment_id)
    .bind(error_message)
    .bind(&now)
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;

    Ok(())
}

fn normalize_portal_base_url(value: &str) -> AppResult<String> {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "Configura la URL del portal de sincronización".to_string(),
        ));
    }
    if !(trimmed.starts_with("https://") || trimmed.starts_with("http://localhost")) {
        return Err(AppError::Validation(
            "La URL del portal debe usar HTTPS, salvo localhost para desarrollo".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

async fn ensure_local_device_ids(db: &SqlitePool, clinic_id: &str) -> AppResult<(String, String)> {
    let license = license_service::get_license_status(db).await?;
    let installation_id = license.installation_id.unwrap_or_else(new_id);
    let device_id = license.device_id.unwrap_or_else(new_id);
    let now = now_utc();
    sqlx::query(
        r#"
        UPDATE app_license
        SET installation_id = COALESCE(installation_id, ?),
            device_id = COALESCE(device_id, ?),
            clinic_id = COALESCE(clinic_id, ?),
            updated_at = ?
        WHERE id = 'local'
        "#,
    )
    .bind(&installation_id)
    .bind(&device_id)
    .bind(clinic_id)
    .bind(&now)
    .execute(db)
    .await?;
    Ok((installation_id, device_id))
}

async fn register_with_portal(
    portal_base_url: &str,
    payload: RegisterInstallationRequest<'_>,
) -> AppResult<RegisterInstallationResponse> {
    let response = Client::new()
        .post(endpoint(portal_base_url, "/register-installation"))
        .json(&payload)
        .send()
        .await
        .map_err(remote_request_error)?;
    read_remote_json(response).await
}

async fn refresh_device_token(
    db: &SqlitePool,
    device: &SyncDeviceRecord,
) -> AppResult<SyncDeviceRecord> {
    let portal_base_url = device.portal_base_url.as_deref().ok_or_else(|| {
        AppError::Validation("El dispositivo no tiene portal configurado".to_string())
    })?;
    let refresh_token = device
        .refresh_token
        .as_deref()
        .ok_or_else(|| AppError::Validation("El dispositivo no tiene refresh token".to_string()))?;
    let response = Client::new()
        .post(endpoint(portal_base_url, "/refresh"))
        .json(&RefreshTokenRequest {
            installation_id: &device.installation_id,
            device_id: &device.device_id,
            refresh_token,
        })
        .send()
        .await
        .map_err(remote_request_error)?;
    let payload: RefreshTokenResponse = read_remote_json(response).await?;
    let now = now_utc();
    sqlx::query(
        r#"
        UPDATE sync_device_tokens
        SET access_token = ?,
            access_token_expires_at = ?,
            refresh_token = ?,
            refresh_token_expires_at = ?,
            status = 'active',
            last_refreshed_at = ?,
            last_error = NULL,
            updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(payload.access_token)
    .bind(payload.token_expires_at)
    .bind(payload.refresh_token)
    .bind(payload.refresh_token_expires_at)
    .bind(&now)
    .bind(&now)
    .bind(&device.id)
    .execute(db)
    .await?;

    active_device(db, &device.clinic_id).await
}

async fn push_outbox(db: &SqlitePool, device: &SyncDeviceRecord) -> AppResult<i64> {
    let rows = sqlx::query_as::<_, OutboxRow>(
        r#"
        SELECT id, aggregate_type, aggregate_id, event_type, payload_json, occurred_at
        FROM sync_outbox
        WHERE clinic_id = ?
          AND status IN ('pending', 'failed')
          AND attempts < ?
          AND available_at <= ?
        ORDER BY occurred_at ASC
        LIMIT 50
        "#,
    )
    .bind(&device.clinic_id)
    .bind(MAX_OUTBOX_ATTEMPTS)
    .bind(now_utc())
    .fetch_all(db)
    .await?;
    if rows.is_empty() {
        return Ok(0);
    }

    let now = now_utc();
    for row in &rows {
        sqlx::query(
            "UPDATE sync_outbox SET status = 'in_flight', attempts = attempts + 1, last_attempt_at = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&now)
        .bind(&now)
        .bind(&row.id)
        .execute(db)
        .await?;
    }

    let events = rows
        .iter()
        .map(|row| {
            let payload = serde_json::from_str::<Value>(&row.payload_json).unwrap_or_else(|_| {
                json!({
                    "raw": row.payload_json,
                    "parseError": true
                })
            });
            PushEvent {
                local_event_id: row.id.clone(),
                aggregate_type: row.aggregate_type.clone(),
                aggregate_id: row.aggregate_id.clone(),
                event_type: row.event_type.clone(),
                payload,
                occurred_at: row.occurred_at.clone(),
            }
        })
        .collect::<Vec<_>>();

    let portal_base_url = required_portal_base_url(device)?;
    let access_token = required_access_token(device)?;
    let response = Client::new()
        .post(endpoint(portal_base_url, "/push"))
        .bearer_auth(access_token)
        .json(&PushRequest { events })
        .send()
        .await
        .map_err(remote_request_error)?;
    let payload: PushResponse = read_remote_json(response).await?;
    let accepted = if payload.accepted_event_ids.is_empty() {
        rows.iter().map(|row| row.id.clone()).collect::<Vec<_>>()
    } else {
        payload.accepted_event_ids
    };
    let synced_at = now_utc();
    let mut pushed = 0;
    for row in &rows {
        if accepted.iter().any(|id| id == &row.id) {
            sqlx::query(
                "UPDATE sync_outbox SET status = 'synced', synced_at = ?, error_message = NULL, updated_at = ? WHERE id = ?",
            )
            .bind(&synced_at)
            .bind(&synced_at)
            .bind(&row.id)
            .execute(db)
            .await?;
            pushed += 1;
        } else {
            sqlx::query(
                r#"
                UPDATE sync_outbox
                SET status = CASE WHEN attempts >= ? THEN 'dead' ELSE 'failed' END,
                    error_message = 'El portal no aceptó el evento',
                    updated_at = ?
                WHERE id = ?
                "#,
            )
            .bind(MAX_OUTBOX_ATTEMPTS)
            .bind(&synced_at)
            .bind(&row.id)
            .execute(db)
            .await?;
        }
    }

    Ok(pushed)
}

async fn pull_and_apply_commands(
    db: &SqlitePool,
    device: &SyncDeviceRecord,
) -> AppResult<(i64, i64)> {
    let portal_base_url = required_portal_base_url(device)?;
    let access_token = required_access_token(device)?;
    let cursor = get_cursor(db, &device.clinic_id, "remote_commands").await?;
    let mut request = Client::new()
        .get(endpoint(portal_base_url, "/pull"))
        .bearer_auth(access_token);
    if let Some(cursor_value) = cursor.as_deref() {
        request = request.query(&[("cursor", cursor_value)]);
    }
    let response = request.send().await.map_err(remote_request_error)?;
    let payload: PullResponse = read_remote_json(response).await?;
    let mut applied = 0;
    let mut failed = 0;

    for command in payload.commands {
        match appointment_service::apply_remote_appointment_status(db, command.clone()).await {
            Ok(_) => applied += 1,
            Err(error) => {
                failed += 1;
                record_failed_remote_command(db, &device.clinic_id, &command, &error.to_string())
                    .await?;
            }
        }
    }

    if let Some(next_cursor) = payload.cursor {
        set_cursor(db, &device.clinic_id, "remote_commands", &next_cursor).await?;
    }
    Ok((applied, failed))
}

async fn ack_remote_receipts(db: &SqlitePool, device: &SyncDeviceRecord) -> AppResult<i64> {
    let rows = sqlx::query_as::<_, ReceiptRow>(
        r#"
        SELECT command_id, status, result_json, error_message, applied_at
        FROM remote_command_receipts
        WHERE clinic_id = ? AND acked_at IS NULL
        ORDER BY created_at ASC
        LIMIT 50
        "#,
    )
    .bind(&device.clinic_id)
    .fetch_all(db)
    .await?;
    if rows.is_empty() {
        return Ok(0);
    }

    let receipts = rows
        .iter()
        .map(|row| AckReceipt {
            command_id: row.command_id.clone(),
            status: row.status.clone(),
            result: row
                .result_json
                .as_deref()
                .and_then(|value| serde_json::from_str::<Value>(value).ok()),
            error_message: row.error_message.clone(),
            applied_at: row.applied_at.clone(),
        })
        .collect::<Vec<_>>();
    let portal_base_url = required_portal_base_url(device)?;
    let access_token = required_access_token(device)?;
    let response = Client::new()
        .post(endpoint(portal_base_url, "/ack"))
        .bearer_auth(access_token)
        .json(&AckRequest { receipts })
        .send()
        .await
        .map_err(remote_request_error)?;
    let payload: AckResponse = read_remote_json(response).await?;
    let acked = if payload.acked_command_ids.is_empty() {
        rows.iter()
            .map(|row| row.command_id.clone())
            .collect::<Vec<_>>()
    } else {
        payload.acked_command_ids
    };
    let now = now_utc();
    for command_id in &acked {
        sqlx::query(
            "UPDATE remote_command_receipts SET acked_at = ?, updated_at = ? WHERE clinic_id = ? AND command_id = ?",
        )
        .bind(&now)
        .bind(&now)
        .bind(&device.clinic_id)
        .bind(command_id)
        .execute(db)
        .await?;
    }
    Ok(acked.len() as i64)
}

async fn sync_status_for_clinic(db: &SqlitePool, clinic_id: &str) -> AppResult<SyncStatus> {
    let devices = sqlx::query_as::<_, SyncDeviceSummary>(
        r#"
        SELECT id, installation_id, device_id, device_label, portal_base_url, status,
               last_registered_at, last_refreshed_at, last_sync_at, last_error,
               revoked_at, updated_at
        FROM sync_device_tokens
        WHERE clinic_id = ?
        ORDER BY updated_at DESC
        "#,
    )
    .bind(clinic_id)
    .fetch_all(db)
    .await?;
    let active_device = devices
        .iter()
        .find(|device| device.status == "active")
        .cloned();
    let pending_outbox = count_scalar(
        db,
        "SELECT COUNT(*) FROM sync_outbox WHERE clinic_id = ? AND status IN ('pending', 'in_flight')",
        clinic_id,
    )
    .await?;
    let failed_outbox = count_scalar(
        db,
        "SELECT COUNT(*) FROM sync_outbox WHERE clinic_id = ? AND status IN ('failed', 'dead')",
        clinic_id,
    )
    .await?;
    let pending_inbox = count_scalar(
        db,
        "SELECT COUNT(*) FROM sync_inbox WHERE clinic_id = ? AND status = 'pending'",
        clinic_id,
    )
    .await?;
    let pending_receipts = count_scalar(
        db,
        "SELECT COUNT(*) FROM remote_command_receipts WHERE clinic_id = ? AND acked_at IS NULL",
        clinic_id,
    )
    .await?;
    let pull_cursor = get_cursor(db, clinic_id, "remote_commands").await?;
    let last_sync_at = active_device
        .as_ref()
        .and_then(|device| device.last_sync_at.clone());
    let last_error = active_device
        .as_ref()
        .and_then(|device| device.last_error.clone());

    Ok(SyncStatus {
        configured: active_device.is_some(),
        active_device,
        devices,
        pending_outbox,
        failed_outbox,
        pending_inbox,
        pending_receipts,
        pull_cursor,
        last_sync_at,
        last_error,
    })
}

async fn count_scalar(db: &SqlitePool, sql: &str, clinic_id: &str) -> AppResult<i64> {
    Ok(sqlx::query_scalar(sql)
        .bind(clinic_id)
        .fetch_one(db)
        .await?)
}

async fn active_device(db: &SqlitePool, clinic_id: &str) -> AppResult<SyncDeviceRecord> {
    sqlx::query_as::<_, SyncDeviceRecord>(
        r#"
        SELECT id, clinic_id, installation_id, device_id, portal_base_url,
               access_token, refresh_token
        FROM sync_device_tokens
        WHERE clinic_id = ? AND status = 'active' AND revoked_at IS NULL
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(clinic_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| {
        AppError::Validation("No hay un dispositivo de sincronización activo".to_string())
    })
}

async fn get_cursor(
    db: &SqlitePool,
    clinic_id: &str,
    cursor_type: &str,
) -> AppResult<Option<String>> {
    Ok(sqlx::query_scalar(
        "SELECT cursor_value FROM sync_cursor WHERE clinic_id = ? AND cursor_type = ?",
    )
    .bind(clinic_id)
    .bind(cursor_type)
    .fetch_optional(db)
    .await?)
}

async fn set_cursor(
    db: &SqlitePool,
    clinic_id: &str,
    cursor_type: &str,
    cursor_value: &str,
) -> AppResult<()> {
    let now = now_utc();
    sqlx::query(
        r#"
        INSERT INTO sync_cursor (id, clinic_id, cursor_type, cursor_value, updated_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(clinic_id, cursor_type) DO UPDATE SET
          cursor_value = excluded.cursor_value,
          updated_at = excluded.updated_at
        "#,
    )
    .bind(new_id())
    .bind(clinic_id)
    .bind(cursor_type)
    .bind(cursor_value)
    .bind(now)
    .execute(db)
    .await?;
    Ok(())
}

async fn remember_device_error(
    db: &SqlitePool,
    device_row_id: &str,
    error_message: &str,
) -> AppResult<()> {
    let now = now_utc();
    sqlx::query("UPDATE sync_device_tokens SET last_error = ?, updated_at = ? WHERE id = ?")
        .bind(error_message)
        .bind(now)
        .bind(device_row_id)
        .execute(db)
        .await?;
    Ok(())
}

fn required_portal_base_url(device: &SyncDeviceRecord) -> AppResult<&str> {
    device
        .portal_base_url
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            AppError::Validation("El dispositivo no tiene portal configurado".to_string())
        })
}

fn required_access_token(device: &SyncDeviceRecord) -> AppResult<&str> {
    device
        .access_token
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError::Validation("El dispositivo no tiene token activo".to_string()))
}

fn endpoint(portal_base_url: &str, path: &str) -> String {
    format!("{portal_base_url}{SYNC_API_PREFIX}{path}")
}

async fn read_remote_json<T: for<'de> Deserialize<'de>>(
    response: reqwest::Response,
) -> AppResult<T> {
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        let summary = body.chars().take(240).collect::<String>();
        let message = format!("Portal de sincronización respondió {status}: {summary}");
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(AppError::Unauthorized);
        }
        return Err(AppError::Conflict(message));
    }
    response.json::<T>().await.map_err(remote_request_error)
}

fn remote_request_error(error: reqwest::Error) -> AppError {
    AppError::Conflict(format!(
        "No se pudo conectar con el portal de sincronización: {error}"
    ))
}

fn is_auth_error(error: &AppError) -> bool {
    matches!(error, AppError::Unauthorized)
}
