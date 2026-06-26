use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use serde_json::{json, Value};
use sqlx::{FromRow, SqlitePool};

use crate::errors::{AppError, AppResult};
use crate::models::LicenseStatus;
use crate::services::audit_service::log_action;
use crate::utils::{new_id, now_utc};

const TRIAL_DAYS: i64 = 30;
const READ_ONLY_MESSAGE: &str = "La licencia está en modo solo lectura. Puedes consultar, exportar información y crear respaldos, pero no modificar datos.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessIntent {
    Read,
    ExportOrBackup,
    DataWrite,
    Restore,
}

#[derive(Debug, FromRow)]
struct LicenseRow {
    trial_started_at: String,
    trial_ends_at: String,
    activated_at: Option<String>,
    status: String,
    access_mode: String,
    last_check_at: Option<String>,
    next_check_at: Option<String>,
    device_id: Option<String>,
    installation_id: Option<String>,
    clinic_id: Option<String>,
    customer_id: Option<String>,
    subscription_id: Option<String>,
    plan_code: Option<String>,
    plan_limits_json: Option<String>,
    grace_period_ends_at: Option<String>,
    offline_grace_until: Option<String>,
    read_only_reason: Option<String>,
}

pub async fn initialize_local_trial(db: &SqlitePool, clinic_id: &str) -> AppResult<()> {
    let now = now_utc();
    let trial_ends_at = (Utc::now() + Duration::days(TRIAL_DAYS)).to_rfc3339();
    sqlx::query(
        r#"
        INSERT INTO app_license
          (id, trial_started_at, trial_ends_at, status, access_mode, last_check_at, next_check_at,
           device_id, installation_id, clinic_id, plan_code, plan_limits_json, updated_at)
        VALUES ('local', ?, ?, 'trial_active', 'full', ?, ?, ?, ?, ?, 'local_trial', ?, ?)
        ON CONFLICT(id) DO UPDATE SET
          trial_started_at = excluded.trial_started_at,
          trial_ends_at = excluded.trial_ends_at,
          activated_at = NULL,
          activated_by_user_id = NULL,
          activation_fingerprint = NULL,
          status = excluded.status,
          access_mode = excluded.access_mode,
          last_check_at = excluded.last_check_at,
          next_check_at = excluded.next_check_at,
          clinic_id = excluded.clinic_id,
          plan_code = excluded.plan_code,
          plan_limits_json = excluded.plan_limits_json,
          read_only_reason = NULL,
          updated_at = excluded.updated_at
        "#,
    )
    .bind(now.clone())
    .bind(&trial_ends_at)
    .bind(now.clone())
    .bind((Utc::now() + Duration::days(1)).to_rfc3339())
    .bind(new_id())
    .bind(new_id())
    .bind(clinic_id)
    .bind(default_plan_limits().to_string())
    .bind(now.clone())
    .execute(db)
    .await?;
    Ok(())
}

pub async fn get_license_status(db: &SqlitePool) -> AppResult<LicenseStatus> {
    ensure_license_row(db).await?;
    let row = sqlx::query_as::<_, LicenseRow>(
        r#"
        SELECT trial_started_at, trial_ends_at, activated_at, status, access_mode,
               last_check_at, next_check_at, device_id, installation_id, clinic_id,
               customer_id, subscription_id, plan_code, plan_limits_json,
               grace_period_ends_at, offline_grace_until, read_only_reason
        FROM app_license
        WHERE id = 'local'
        "#,
    )
    .fetch_optional(db)
    .await?;

    Ok(row
        .map(|license| build_license_status(&license))
        .unwrap_or_else(not_configured_status))
}

pub async fn ensure_license_allows(
    db: &SqlitePool,
    intent: AccessIntent,
) -> AppResult<LicenseStatus> {
    let license = get_license_status(db).await?;
    let allowed = match intent {
        AccessIntent::Read | AccessIntent::ExportOrBackup => true,
        AccessIntent::DataWrite | AccessIntent::Restore => license.can_write,
    };
    if allowed {
        return Ok(license);
    }

    let action = match intent {
        AccessIntent::Restore => "license.block_restore",
        AccessIntent::DataWrite => "license.block_write",
        AccessIntent::Read => "license.read",
        AccessIntent::ExportOrBackup => "license.export",
    };
    log_action(
        db,
        license.clinic_id.as_deref(),
        None,
        action,
        "app_license",
        Some("local"),
        "warning",
        Some(json!({ "status": license.status, "accessMode": license.access_mode })),
    )
    .await?;

    Err(AppError::Conflict(license.message))
}

async fn ensure_license_row(db: &SqlitePool) -> AppResult<()> {
    let existing_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM app_license")
        .fetch_one(db)
        .await?;
    if existing_count > 0 {
        return Ok(());
    }

    let clinic: Option<(String, String)> =
        sqlx::query_as("SELECT id, created_at FROM clinics ORDER BY created_at LIMIT 1")
            .fetch_optional(db)
            .await?;
    let Some((clinic_id, trial_started_at)) = clinic else {
        return Ok(());
    };

    let start = parse_license_datetime(&trial_started_at).unwrap_or_else(Utc::now);
    let trial_ends_at = (start + Duration::days(TRIAL_DAYS)).to_rfc3339();
    let now = now_utc();
    sqlx::query(
        r#"
        INSERT INTO app_license
          (id, trial_started_at, trial_ends_at, status, access_mode, last_check_at, next_check_at,
           device_id, installation_id, clinic_id, plan_code, plan_limits_json, updated_at)
        VALUES ('local', ?, ?, 'trial_active', 'full', ?, ?, ?, ?, ?, 'local_trial', ?, ?)
        "#,
    )
    .bind(trial_started_at)
    .bind(trial_ends_at)
    .bind(now.clone())
    .bind((Utc::now() + Duration::days(1)).to_rfc3339())
    .bind(new_id())
    .bind(new_id())
    .bind(clinic_id)
    .bind(default_plan_limits().to_string())
    .bind(now.clone())
    .execute(db)
    .await?;
    Ok(())
}

fn build_license_status(row: &LicenseRow) -> LicenseStatus {
    let now = Utc::now();
    let trial_ends_at = parse_license_datetime(&row.trial_ends_at);
    let grace_until = row
        .grace_period_ends_at
        .as_deref()
        .and_then(parse_license_datetime);
    let offline_until = row
        .offline_grace_until
        .as_deref()
        .and_then(parse_license_datetime);
    let is_legacy_active = row.activated_at.is_some();
    let trial_expired = !is_legacy_active && trial_ends_at.map(|end| now >= end).unwrap_or(true);

    let mut status = row.status.trim().to_string();
    if is_legacy_active
        && matches!(
            status.as_str(),
            "" | "licensed" | "trial_active" | "expired"
        )
    {
        status = "active".to_string();
    } else if trial_expired && matches!(status.as_str(), "" | "trial_active" | "expired") {
        status = "expired".to_string();
    } else if status.is_empty() {
        status = "trial_active".to_string();
    }

    let access_mode = if matches!(status.as_str(), "active" | "trial_active")
        || (status == "grace_period" && grace_until.map(|end| now < end).unwrap_or(true))
        || (status == "offline_grace" && offline_until.map(|end| now < end).unwrap_or(true))
    {
        "full"
    } else {
        "read_only"
    };

    let stored_mode = row.access_mode.trim();
    let access_mode = if stored_mode == "read_only" {
        "read_only"
    } else {
        access_mode
    };
    let can_write = access_mode == "full";
    let seconds_remaining = trial_ends_at
        .map(|end| (end - now).num_seconds().max(0))
        .unwrap_or(0);
    let days_remaining = if seconds_remaining == 0 {
        0
    } else {
        (seconds_remaining + 86_399) / 86_400
    };
    let is_trial_active = status == "trial_active" && can_write;
    let is_expired = matches!(
        status.as_str(),
        "expired" | "suspended" | "past_due" | "read_only"
    );
    let is_licensed =
        matches!(status.as_str(), "active" | "grace_period" | "offline_grace") || is_legacy_active;
    let message = license_message(&status, can_write, row.read_only_reason.as_deref());
    let plan_limits = row
        .plan_limits_json
        .as_deref()
        .and_then(|value| serde_json::from_str::<Value>(value).ok())
        .or_else(|| Some(default_plan_limits()));

    LicenseStatus {
        status,
        access_mode: access_mode.to_string(),
        can_write,
        message,
        trial_started_at: Some(row.trial_started_at.clone()),
        trial_ends_at: Some(row.trial_ends_at.clone()),
        activated_at: row.activated_at.clone(),
        last_check_at: row.last_check_at.clone(),
        next_check_at: row.next_check_at.clone(),
        device_id: row.device_id.clone(),
        installation_id: row.installation_id.clone(),
        clinic_id: row.clinic_id.clone(),
        customer_id: row.customer_id.clone(),
        subscription_id: row.subscription_id.clone(),
        plan_code: row.plan_code.clone(),
        plan_limits,
        days_remaining,
        is_trial_active,
        is_expired,
        is_licensed,
        requires_activation: false,
    }
}

fn not_configured_status() -> LicenseStatus {
    LicenseStatus {
        status: "not_configured".to_string(),
        access_mode: "setup".to_string(),
        can_write: false,
        message: "Licencia local pendiente de configuración inicial".to_string(),
        trial_started_at: None,
        trial_ends_at: None,
        activated_at: None,
        last_check_at: None,
        next_check_at: None,
        device_id: None,
        installation_id: None,
        clinic_id: None,
        customer_id: None,
        subscription_id: None,
        plan_code: None,
        plan_limits: None,
        days_remaining: 0,
        is_trial_active: false,
        is_expired: false,
        is_licensed: false,
        requires_activation: false,
    }
}

fn license_message(status: &str, can_write: bool, reason: Option<&str>) -> String {
    if can_write {
        return match status {
            "trial_active" => {
                "Prueba local activa. Puedes trabajar sin internet con todas las funciones locales."
                    .to_string()
            }
            "grace_period" => {
                "Licencia en periodo de gracia. El sistema sigue habilitado temporalmente; regulariza la licencia para evitar modo sólo lectura."
                    .to_string()
            }
            "offline_grace" => {
                "No se pudo validar en línea, pero tu gracia offline mantiene la operación completa."
                    .to_string()
            }
            "active" => "Licencia activa para operación local profesional.".to_string(),
            _ => "Licencia habilitada para operación local.".to_string(),
        };
    }

    match status {
        "expired" => {
            return "La prueba o licencia terminó. El sistema queda en modo sólo lectura para proteger tus datos; puedes consultar, exportar y crear respaldos."
                .to_string();
        }
        "past_due" => {
            return "La licencia tiene pagos pendientes. El sistema queda en modo sólo lectura; puedes consultar, exportar y crear respaldos."
                .to_string();
        }
        "suspended" => {
            return "La licencia fue suspendida. El sistema queda en modo sólo lectura; puedes consultar, exportar y crear respaldos."
                .to_string();
        }
        "read_only" => {
            return "El sistema está en modo sólo lectura. Puedes consultar, exportar información y crear respaldos, pero no modificar datos."
                .to_string();
        }
        "not_configured" => {
            return "Licencia local pendiente de configuración inicial.".to_string();
        }
        _ => {}
    }

    match reason {
        Some("trial_expired") => {
            "La prueba terminó. El sistema quedó en modo solo lectura para proteger tus datos."
                .to_string()
        }
        Some(value) if !value.is_empty() => format!("{READ_ONLY_MESSAGE} Motivo: {value}."),
        _ => READ_ONLY_MESSAGE.to_string(),
    }
}

fn parse_license_datetime(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|date| Utc.from_utc_datetime(&date))
        })
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S")
                .ok()
                .map(|date| Utc.from_utc_datetime(&date))
        })
}

fn default_plan_limits() -> Value {
    json!({
        "patients": null,
        "users": null,
        "storageMb": null,
        "mode": "local"
    })
}

#[cfg(test)]
pub async fn dev_activate_legacy_local_license(
    db: &SqlitePool,
    user_id: Option<&str>,
) -> AppResult<()> {
    ensure_license_row(db).await?;
    let now = now_utc();
    sqlx::query(
        r#"
        UPDATE app_license
        SET status = 'active',
            access_mode = 'full',
            activated_at = COALESCE(activated_at, ?),
            activated_by_user_id = COALESCE(activated_by_user_id, ?),
            activation_fingerprint = ?,
            read_only_reason = NULL,
            updated_at = ?
        WHERE id = 'local'
        "#,
    )
    .bind(now.clone())
    .bind(user_id)
    .bind(new_id())
    .bind(now.clone())
    .execute(db)
    .await?;
    Ok(())
}
