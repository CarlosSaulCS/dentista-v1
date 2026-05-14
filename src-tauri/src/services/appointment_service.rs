use chrono::{Duration, Local, NaiveDateTime};
use serde_json::json;
use sqlx::SqlitePool;

use crate::errors::{AppError, AppResult};
use crate::models::{
    AppointmentSummary, CreateAppointmentInput, ListAppointmentsInput, UpdateAppointmentInput,
    UpdateAppointmentStatusInput,
};
use crate::services::audit_service::log_action;
use crate::services::auth_service::{validate_session, validate_session_for_intent};
use crate::services::license_service::AccessIntent;
use crate::utils::{new_id, now_utc, today_prefix};

pub async fn create_appointment(
    db: &SqlitePool,
    session_token: &str,
    input: CreateAppointmentInput,
) -> AppResult<AppointmentSummary> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("appointments.create"),
        AccessIntent::DataWrite,
    )
    .await?;
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
    ensure_no_conflicting_appointment(
        db,
        &ctx.clinic_id,
        None,
        &input.patient_id,
        input
            .dentist_user_id
            .as_deref()
            .filter(|value| !value.is_empty()),
        &starts_at,
        &ends_at,
    )
    .await?;
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
        SELECT a.id, a.patient_id, p.full_name AS patient_name, p.phone AS patient_phone,
               p.whatsapp AS patient_whatsapp, p.email AS patient_email, a.dentist_user_id,
               u.full_name AS dentist_name, a.starts_at, a.ends_at, a.duration_minutes,
               a.reason, a.appointment_type, a.status, a.notes
        FROM appointments a
        JOIN patients p ON p.id = a.patient_id
        LEFT JOIN users u ON u.id = a.dentist_user_id
        WHERE a.clinic_id = ?
          AND a.deleted_at IS NULL
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

pub async fn get_next_patient_appointment(
    db: &SqlitePool,
    session_token: &str,
    patient_id: &str,
) -> AppResult<Option<AppointmentSummary>> {
    let ctx = validate_session(db, session_token, None).await?;
    if patient_id.trim().is_empty() {
        return Err(AppError::Validation("Selecciona un paciente".to_string()));
    }

    let now = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let appointment = sqlx::query_as::<_, AppointmentSummary>(
        r#"
        SELECT a.id, a.patient_id, p.full_name AS patient_name, p.phone AS patient_phone,
               p.whatsapp AS patient_whatsapp, p.email AS patient_email, a.dentist_user_id,
               u.full_name AS dentist_name, a.starts_at, a.ends_at, a.duration_minutes,
               a.reason, a.appointment_type, a.status, a.notes
        FROM appointments a
        JOIN patients p ON p.id = a.patient_id
        LEFT JOIN users u ON u.id = a.dentist_user_id
        WHERE a.clinic_id = ?
          AND a.patient_id = ?
          AND a.deleted_at IS NULL
          AND a.starts_at >= ?
          AND a.status NOT IN ('cancelada', 'finalizada', 'no_asistio')
        ORDER BY a.starts_at
        LIMIT 1
        "#,
    )
    .bind(&ctx.clinic_id)
    .bind(patient_id.trim())
    .bind(now)
    .fetch_optional(db)
    .await?;

    Ok(appointment)
}

pub async fn update_appointment(
    db: &SqlitePool,
    session_token: &str,
    input: UpdateAppointmentInput,
) -> AppResult<AppointmentSummary> {
    let required_permission = if input.status == "cancelada" {
        "appointments.cancel"
    } else {
        "appointments.reschedule"
    };
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some(required_permission),
        AccessIntent::DataWrite,
    )
    .await?;
    if input.id.trim().is_empty()
        || input.patient_id.trim().is_empty()
        || input.reason.trim().is_empty()
    {
        return Err(AppError::Validation(
            "Cita, paciente y motivo son obligatorios".to_string(),
        ));
    }
    if input.duration_minutes < 5 || input.duration_minutes > 480 {
        return Err(AppError::Validation(
            "La duración debe estar entre 5 y 480 minutos".to_string(),
        ));
    }
    ensure_valid_appointment_status(input.status.trim())?;

    let previous_status: String = sqlx::query_scalar(
        "SELECT status FROM appointments WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL",
    )
    .bind(&ctx.clinic_id)
    .bind(input.id.trim())
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::Validation("Cita no encontrada".to_string()))?;
    let starts = parse_local_datetime(&input.starts_at)?;
    let ends = starts + Duration::minutes(input.duration_minutes);
    let starts_at = starts.format("%Y-%m-%dT%H:%M:%S").to_string();
    let ends_at = ends.format("%Y-%m-%dT%H:%M:%S").to_string();
    let dentist_user_id = input
        .dentist_user_id
        .as_deref()
        .filter(|value| !value.is_empty());
    ensure_no_conflicting_appointment(
        db,
        &ctx.clinic_id,
        Some(input.id.trim()),
        &input.patient_id,
        dentist_user_id,
        &starts_at,
        &ends_at,
    )
    .await?;

    let now = now_utc();
    sqlx::query(
        r#"
        UPDATE appointments
        SET patient_id = ?, dentist_user_id = ?, starts_at = ?, ends_at = ?,
            duration_minutes = ?, reason = ?, appointment_type = ?, status = ?,
            notes = ?, updated_at = ?, cancelled_at = CASE WHEN ? = 'cancelada' THEN ? ELSE NULL END
        WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL
        "#,
    )
    .bind(input.patient_id.trim())
    .bind(dentist_user_id)
    .bind(&starts_at)
    .bind(&ends_at)
    .bind(input.duration_minutes)
    .bind(input.reason.trim())
    .bind(input.appointment_type.trim())
    .bind(input.status.trim())
    .bind(input.notes.as_deref().map(str::trim))
    .bind(&now)
    .bind(input.status.trim())
    .bind(&now)
    .bind(&ctx.clinic_id)
    .bind(input.id.trim())
    .execute(db)
    .await?;

    insert_event(
        db,
        &ctx.clinic_id,
        input.id.trim(),
        &ctx.user_id,
        if previous_status == input.status {
            "updated"
        } else {
            "status_changed"
        },
        Some(&previous_status),
        Some(input.status.trim()),
        input.notes.as_deref(),
    )
    .await?;
    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "appointments.update",
        "appointments",
        Some(input.id.trim()),
        "info",
        Some(
            json!({ "patientId": input.patient_id, "startsAt": starts_at, "status": input.status }),
        ),
    )
    .await?;

    get_appointment_by_id(db, &ctx.clinic_id, input.id.trim()).await
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
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some(required_permission),
        AccessIntent::DataWrite,
    )
    .await?;
    ensure_valid_appointment_status(input.status.trim())?;

    let previous_status: String = sqlx::query_scalar(
        "SELECT status FROM appointments WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL",
    )
    .bind(&ctx.clinic_id)
    .bind(&input.appointment_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::Validation("Cita no encontrada".to_string()))?;

    let now = now_utc();
    sqlx::query("UPDATE appointments SET status = ?, notes = COALESCE(?, notes), updated_at = ?, cancelled_at = CASE WHEN ? = 'cancelada' THEN ? ELSE cancelled_at END WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL")
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

pub async fn soft_delete_appointment(
    db: &SqlitePool,
    session_token: &str,
    appointment_id: &str,
) -> AppResult<AppointmentSummary> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("appointments.cancel"),
        AccessIntent::DataWrite,
    )
    .await?;
    if appointment_id.trim().is_empty() {
        return Err(AppError::Validation(
            "Selecciona una cita para eliminar".to_string(),
        ));
    }
    let appointment = get_appointment_by_id(db, &ctx.clinic_id, appointment_id).await?;
    let now = now_utc();
    sqlx::query(
        "UPDATE appointments SET status = 'cancelada', deleted_by_user_id = ?, cancelled_at = COALESCE(cancelled_at, ?), updated_at = ? WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL",
    )
    .bind(&ctx.user_id)
    .bind(&now)
    .bind(&now)
    .bind(&ctx.clinic_id)
    .bind(appointment_id.trim())
    .execute(db)
    .await?;

    insert_event(
        db,
        &ctx.clinic_id,
        appointment_id.trim(),
        &ctx.user_id,
        "soft_deleted",
        Some(&appointment.status),
        Some("cancelada"),
        Some("Baja lógica de cita"),
    )
    .await?;
    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "appointments.soft_delete",
        "appointments",
        Some(appointment_id.trim()),
        "warning",
        Some(json!({ "patientId": appointment.patient_id, "startsAt": appointment.starts_at })),
    )
    .await?;

    Ok(appointment)
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
        SELECT a.id, a.patient_id, p.full_name AS patient_name, p.phone AS patient_phone,
               p.whatsapp AS patient_whatsapp, p.email AS patient_email, a.dentist_user_id,
               u.full_name AS dentist_name, a.starts_at, a.ends_at, a.duration_minutes,
               a.reason, a.appointment_type, a.status, a.notes
        FROM appointments a
        JOIN patients p ON p.id = a.patient_id
        LEFT JOIN users u ON u.id = a.dentist_user_id
        WHERE a.clinic_id = ? AND a.id = ? AND a.deleted_at IS NULL
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

fn ensure_valid_appointment_status(status: &str) -> AppResult<()> {
    if matches!(
        status,
        "programada"
            | "confirmada"
            | "en_espera"
            | "en_consulta"
            | "finalizada"
            | "cancelada"
            | "no_asistio"
    ) {
        return Ok(());
    }
    Err(AppError::Validation(
        "Estado de cita no soportado".to_string(),
    ))
}

async fn ensure_no_conflicting_appointment(
    db: &SqlitePool,
    clinic_id: &str,
    except_id: Option<&str>,
    patient_id: &str,
    dentist_user_id: Option<&str>,
    starts_at: &str,
    ends_at: &str,
) -> AppResult<()> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM appointments
        WHERE clinic_id = ?
          AND deleted_at IS NULL
          AND status NOT IN ('cancelada', 'no_asistio', 'finalizada')
          AND (? IS NULL OR id <> ?)
          AND starts_at < ?
          AND ends_at > ?
          AND (patient_id = ? OR (? IS NOT NULL AND dentist_user_id = ?))
        "#,
    )
    .bind(clinic_id)
    .bind(except_id)
    .bind(except_id)
    .bind(ends_at)
    .bind(starts_at)
    .bind(patient_id.trim())
    .bind(dentist_user_id)
    .bind(dentist_user_id)
    .fetch_one(db)
    .await?;

    if count > 0 {
        return Err(AppError::Validation(
            "Ya existe una cita activa en ese horario para el paciente o responsable seleccionado"
                .to_string(),
        ));
    }
    Ok(())
}
