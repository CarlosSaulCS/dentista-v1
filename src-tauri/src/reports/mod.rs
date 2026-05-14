//! Generación y registro de reportes locales.
//!
//! Los reportes se guardan desde Rust para que funcionen igual en Tauri que en
//! Windows instalado, sin depender de descargas del WebView.

use std::fs;
use std::path::PathBuf;

use chrono::Local;
use serde_json::json;

use crate::database::AppState;
use crate::errors::{AppError, AppResult};
use crate::models::{ReportExportResult, SaveReportFileInput};
use crate::services::audit_service::log_action;
use crate::services::auth_service::validate_session_for_intent;
use crate::services::license_service::AccessIntent;
use crate::utils::{new_id, now_utc};

pub async fn save_report_file(
    state: &AppState,
    session_token: &str,
    input: SaveReportFileInput,
) -> AppResult<ReportExportResult> {
    let ctx = validate_session_for_intent(
        &state.db,
        session_token,
        Some("reports.financial"),
        AccessIntent::ExportOrBackup,
    )
    .await?;
    if input.bytes.is_empty() {
        return Err(AppError::Validation(
            "El reporte no contiene datos para guardar".to_string(),
        ));
    }

    let format = input.format.trim().to_lowercase();
    if !matches!(format.as_str(), "pdf" | "csv" | "xlsx") {
        return Err(AppError::Validation(
            "Formato de reporte no soportado".to_string(),
        ));
    }

    let report_type = input.report_type.trim();
    if report_type.is_empty() {
        return Err(AppError::Validation(
            "Tipo de reporte obligatorio".to_string(),
        ));
    }

    fs::create_dir_all(&state.reports_dir)?;
    let safe_name = normalize_report_name(&input.file_name, &format);
    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let path = input
        .target_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| normalize_target_path(value, &format))
        .unwrap_or_else(|| state.reports_dir.join(format!("{timestamp}-{safe_name}")));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, &input.bytes)?;

    let id = new_id();
    let created_at = now_utc();
    let size_bytes = input.bytes.len() as u64;
    let path_text = path.to_string_lossy().to_string();

    sqlx::query(
        r#"
        INSERT INTO report_exports
          (id, clinic_id, report_type, format, filters_json, file_id, created_by_user_id,
           path, size_bytes, created_at)
        VALUES (?, ?, ?, ?, ?, NULL, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&ctx.clinic_id)
    .bind(report_type)
    .bind(&format)
    .bind(input.filters_json.as_deref())
    .bind(&ctx.user_id)
    .bind(&path_text)
    .bind(size_bytes as i64)
    .bind(&created_at)
    .execute(&state.db)
    .await?;

    log_action(
        &state.db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "reports.export",
        "report_exports",
        Some(&id),
        "info",
        Some(json!({
            "reportType": report_type,
            "format": format,
            "path": path_text,
            "sizeBytes": size_bytes
        })),
    )
    .await?;

    Ok(ReportExportResult {
        id,
        path: path_text,
        size_bytes,
        created_at,
    })
}

fn normalize_report_name(file_name: &str, format: &str) -> String {
    let fallback = format!("reporte-dentalcare.{format}");
    let trimmed = file_name.trim();
    let raw = if trimmed.is_empty() {
        fallback.as_str()
    } else {
        trimmed
    };
    let sanitized = sanitize_filename::sanitize(raw);
    if sanitized.to_lowercase().ends_with(&format!(".{format}")) {
        sanitized
    } else {
        format!("{sanitized}.{format}")
    }
}

fn normalize_target_path(path: &str, format: &str) -> PathBuf {
    let mut path = PathBuf::from(path.trim());
    let has_expected_extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case(format))
        .unwrap_or(false);
    if !has_expected_extension {
        path.set_extension(format);
    }
    path
}
