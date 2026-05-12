use chrono::{Duration, NaiveDateTime};
use serde_json::json;
use sqlx::SqlitePool;

use crate::errors::{AppError, AppResult};
use crate::models::{
    AppointmentSummary, CreateAppointmentInput, ListAppointmentsInput, UpdateAppointmentStatusInput,
};
use crate::services::audit_service::log_action;
use crate::services::auth_service::validate_session;
use crate::utils::{new_id, now_utc, today_prefix};

pub async fn create_appointment(
    db: &SqlitePool,
    session_token: &str,
    input: CreateAppointmentInput,
) -> AppResult<AppointmentSummary> {
    let ctx = validate_session(db, session_token, Some("appointments.create")).await?;
    if input.patient_id.trim().is_empty() || input.reason.trim().is_empty() {
        return Err(AppError::Validation(
            "Paciente y motivo son obligatorios".to_string(),
        ));
    }
    if input.duration_minutes < 5 || input.duration_minutes > 480 {
        return Err(AppError::Validation(
            "La duración debe estar entre 5 y 480 minutos".to_string(),
        ));
    }

    let starts = parse_local_datetime(&input.starts_at)?;
    let ends = starts + Duration::minutes(input.duration_minutes);
    let starts_at = starts.format("%Y-%m-%dT%H:%M:%S").to_string();
    let ends_at = ends.format("%Y-%m-%dT%H:%M:%S").to_string();
    let id = new_id();
    let now = now_utc();

    sqlx::query(
        r#"
        INSERT INTO appointments
          (id, clinic_id, patient_id, dentist_user_id, starts_at, ends_at, duration_minutes,
           reason, appointment_type, status, notes, created_by_user_id, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'programada', ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&ctx.clinic_id)
    .bind(&input.patient_id)
    .bind(
        input
            .dentist_user_id
            .as_deref()
            .filter(|value| !value.is_empty()),
    )
    .bind(&starts_at)
    .bind(&ends_at)
    .bind(input.duration_minutes)
    .bind(input.reason.trim())
    .bind(input.appointment_type.trim())
    .bind(input.notes.as_deref().map(str::trim))
    .bind(&ctx.user_id)
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;

    insert_event(
        db,
        &ctx.clinic_id,
        &id,
        &ctx.user_id,
        "created",
        None,
        Some("programada"),
        None,
    )
    .await?;
    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "appointments.create",
        "appointments",
        Some(&id),
        "info",
        Some(json!({ "patientId": input.patient_id, "startsAt": starts_at })),
    )
    .await?;

    get_appointment_by_id(db, &ctx.clinic_id, &id).await
}

pub async fn list_appointments(
    db: &SqlitePool,
    session_token: &str,
    input: ListAppointmentsInput,
) -> AppResult<Vec<AppointmentSummary>> {
    let ctx = validate_session(db, session_token, None).await?;
    let date = input.date.unwrap_or_else(today_prefix);
    let status = input.status.unwrap_or_default();
    let date_search = format!("{date}%");

    let appointments = sqlx::query_as::<_, AppointmentSummary>(
        r#"
        SELECT a.id, a.patient_id, p.full_name AS patient_name, a.dentist_user_id,
               u.full_name AS dentist_name, a.starts_at, a.ends_at, a.duration_minutes,
               a.reason, a.appointment_type, a.status, a.notes
        FROM appointments a
        JOIN patients p ON p.id = a.patient_id
        LEFT JOIN users u ON u.id = a.dentist_user_id
        WHERE a.clinic_id = ?
          AND a.starts_at LIKE ?
          AND (? = '' OR a.status = ?)
        ORDER BY a.starts_at
        "#,
    )
    .bind(ctx.clinic_id)
    .bind(date_search)
    .bind(&status)
    .bind(&status)
    .fetch_all(db)
    .await?;

    Ok(appointments)
}

pub async fn update_appointment_status(
    db: &SqlitePool,
    session_token: &str,
    input: UpdateAppointmentStatusInput,
) -> AppResult<AppointmentSummary> {
    let required_permission = if input.status == "cancelada" {
        "appointments.cancel"
    } else if input.status == "reprogramada" {
        "appointments.reschedule"
    } else {
        "appointments.create"
    };
    let ctx = validate_session(db, session_token, Some(required_permission)).await?;

    let previous_status: String =
        sqlx::query_scalar("SELECT status FROM appointments WHERE clinic_id = ? AND id = ?")
            .bind(&ctx.clinic_id)
            .bind(&input.appointment_id)
            .fetch_optional(db)
            .await?
            .ok_or_else(|| AppError::Validation("Cita no encontrada".to_string()))?;

    let now = now_utc();
    sqlx::query("UPDATE appointments SET status = ?, notes = COALESCE(?, notes), updated_at = ?, cancelled_at = CASE WHEN ? = 'cancelada' THEN ? ELSE cancelled_at END WHERE clinic_id = ? AND id = ?")
        .bind(&input.status)
        .bind(input.notes.as_deref().map(str::trim))
        .bind(&now)
        .bind(&input.status)
        .bind(&now)
        .bind(&ctx.clinic_id)
        .bind(&input.appointment_id)
        .execute(db)
        .await?;

    insert_event(
        db,
        &ctx.clinic_id,
        &input.appointment_id,
        &ctx.user_id,
        "status_changed",
        Some(&previous_status),
        Some(&input.status),
        input.notes.as_deref(),
    )
    .await?;
    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "appointments.status_changed",
        "appointments",
        Some(&input.appointment_id),
        "info",
        Some(json!({ "from": previous_status, "to": input.status })),
    )
    .await?;

    get_appointment_by_id(db, &ctx.clinic_id, &input.appointment_id).await
}

#[expect(
    clippy::too_many_arguments,
    reason = "Appointment events map directly to the event table fields."
)]
async fn insert_event(
    db: &SqlitePool,
    clinic_id: &str,
    appointment_id: &str,
    user_id: &str,
    event_type: &str,
    from_status: Option<&str>,
    to_status: Option<&str>,
    note: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO appointment_events
          (id, clinic_id, appointment_id, user_id, event_type, from_status, to_status, metadata, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(new_id())
    .bind(clinic_id)
    .bind(appointment_id)
    .bind(user_id)
    .bind(event_type)
    .bind(from_status)
    .bind(to_status)
    .bind(note.map(|value| json!({ "note": value }).to_string()))
    .bind(now_utc())
    .execute(db)
    .await?;
    Ok(())
}

async fn get_appointment_by_id(
    db: &SqlitePool,
    clinic_id: &str,
    appointment_id: &str,
) -> AppResult<AppointmentSummary> {
    sqlx::query_as::<_, AppointmentSummary>(
        r#"
        SELECT a.id, a.patient_id, p.full_name AS patient_name, a.dentist_user_id,
               u.full_name AS dentist_name, a.starts_at, a.ends_at, a.duration_minutes,
               a.reason, a.appointment_type, a.status, a.notes
        FROM appointments a
        JOIN patients p ON p.id = a.patient_id
        LEFT JOIN users u ON u.id = a.dentist_user_id
        WHERE a.clinic_id = ? AND a.id = ?
        "#,
    )
    .bind(clinic_id)
    .bind(appointment_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::Validation("Cita no encontrada".to_string()))
}

fn parse_local_datetime(value: &str) -> AppResult<NaiveDateTime> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M"))
        .map_err(|_| AppError::Validation("Fecha y hora de cita inválidas".to_string()))
}
