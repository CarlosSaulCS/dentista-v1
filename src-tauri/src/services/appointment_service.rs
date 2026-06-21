use chrono::{Duration, Local, NaiveDateTime};
use serde_json::json;
use sqlx::SqlitePool;

use crate::errors::{AppError, AppResult};
use crate::models::{
    AppointmentSummary, CreateAppointmentInput, ListAppointmentsInput,
    RemoteAppointmentStatusCommand, RemoteCommandApplyResult, UpdateAppointmentInput,
    UpdateAppointmentStatusInput,
};
use crate::services::audit_service::log_action;
use crate::services::auth_service::{validate_session, validate_session_for_intent};
use crate::services::license_service::AccessIntent;
use crate::services::sync_service;
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
        Some(&ctx.user_id),
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

    let appointment = get_appointment_by_id(db, &ctx.clinic_id, &id).await?;
    sync_service::enqueue_local_event(
        db,
        &ctx.clinic_id,
        "appointments",
        &id,
        "appointment.created",
        json!({ "appointment": appointment.clone() }),
    )
    .await?;

    Ok(appointment)
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
        Some(&ctx.user_id),
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

    let appointment = get_appointment_by_id(db, &ctx.clinic_id, input.id.trim()).await?;
    sync_service::enqueue_local_event(
        db,
        &ctx.clinic_id,
        "appointments",
        input.id.trim(),
        "appointment.updated",
        json!({
            "appointment": appointment.clone(),
            "previousStatus": previous_status
        }),
    )
    .await?;

    Ok(appointment)
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
    apply_appointment_status_update(
        db,
        &ctx.clinic_id,
        Some(&ctx.user_id),
        input,
        "appointments.status_changed",
        json!({ "source": "local" }),
        true,
    )
    .await
}

pub async fn apply_remote_appointment_status(
    db: &SqlitePool,
    command: RemoteAppointmentStatusCommand,
) -> AppResult<RemoteCommandApplyResult> {
    if command.command_id.trim().is_empty() || command.appointment_id.trim().is_empty() {
        return Err(AppError::Validation(
            "Comando remoto de cita incompleto".to_string(),
        ));
    }
    if !matches!(
        command.command_type.as_str(),
        "confirm_appointment" | "cancel_appointment"
    ) {
        return Err(AppError::Validation(
            "Comando remoto de cita no soportado".to_string(),
        ));
    }

    let clinic_id: String = sqlx::query_scalar(
        "SELECT clinic_id FROM appointments WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(command.appointment_id.trim())
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::Validation("Cita no encontrada".to_string()))?;
    if command
        .clinic_id
        .as_deref()
        .filter(|remote_clinic_id| !remote_clinic_id.is_empty() && *remote_clinic_id != clinic_id)
        .is_some()
    {
        return Err(AppError::Validation(
            "El comando remoto no corresponde al consultorio local".to_string(),
        ));
    }

    if let Some((status, result_json, applied_at)) = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
        "SELECT status, result_json, applied_at FROM remote_command_receipts WHERE clinic_id = ? AND command_id = ?",
    )
    .bind(&clinic_id)
    .bind(command.command_id.trim())
    .fetch_optional(db)
    .await?
    {
        if status == "applied" {
            return Ok(RemoteCommandApplyResult {
                command_id: command.command_id,
                appointment_id: command.appointment_id,
                status,
                applied_at,
                message: result_json,
            });
        }
    }

    let now = now_utc();
    sqlx::query(
        r#"
        INSERT INTO sync_inbox
          (id, clinic_id, command_id, command_type, aggregate_type, aggregate_id,
           payload_json, requested_by_json, received_at, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, 'appointments', ?, ?, ?, ?, 'pending', ?, ?)
        ON CONFLICT(clinic_id, command_id) DO UPDATE SET
          status = CASE WHEN sync_inbox.status = 'applied' THEN sync_inbox.status ELSE 'pending' END,
          updated_at = excluded.updated_at
        "#,
    )
    .bind(new_id())
    .bind(&clinic_id)
    .bind(command.command_id.trim())
    .bind(command.command_type.trim())
    .bind(command.appointment_id.trim())
    .bind(command.payload.to_string())
    .bind(
        command
            .requested_by
            .as_ref()
            .map(|requested_by| json!(requested_by).to_string()),
    )
    .bind(&now)
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;

    let target_status = command
        .payload
        .get("targetStatus")
        .and_then(|value| value.as_str())
        .unwrap_or_else(|| {
            if command.command_type == "cancel_appointment" {
                "cancelada"
            } else {
                "confirmada"
            }
        })
        .trim()
        .to_string();
    let note = command
        .payload
        .get("note")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let appointment = apply_appointment_status_update(
        db,
        &clinic_id,
        None,
        UpdateAppointmentStatusInput {
            appointment_id: command.appointment_id.clone(),
            status: target_status,
            notes: note,
        },
        "appointments.remote_status_applied",
        json!({
            "source": "dv1_bridge",
            "commandId": command.command_id.clone(),
            "commandType": command.command_type.clone(),
            "requestedBy": command.requested_by.clone()
        }),
        false,
    )
    .await?;
    let applied_at = now_utc();
    let result_json = json!({
        "appointmentId": appointment.id,
        "status": appointment.status,
        "updatedAt": applied_at
    })
    .to_string();

    sqlx::query(
        r#"
        UPDATE sync_inbox
        SET status = 'applied', applied_at = ?, error_message = NULL, updated_at = ?
        WHERE clinic_id = ? AND command_id = ?
        "#,
    )
    .bind(&applied_at)
    .bind(&applied_at)
    .bind(&clinic_id)
    .bind(command.command_id.trim())
    .execute(db)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO remote_command_receipts
          (id, clinic_id, command_id, command_type, appointment_id, status, result_json,
           received_at, applied_at, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, 'applied', ?, ?, ?, ?, ?)
        ON CONFLICT(clinic_id, command_id) DO UPDATE SET
          status = 'applied',
          result_json = excluded.result_json,
          error_message = NULL,
          applied_at = excluded.applied_at,
          updated_at = excluded.updated_at
        "#,
    )
    .bind(new_id())
    .bind(&clinic_id)
    .bind(command.command_id.trim())
    .bind(command.command_type.trim())
    .bind(command.appointment_id.trim())
    .bind(&result_json)
    .bind(&now)
    .bind(&applied_at)
    .bind(&now)
    .bind(&applied_at)
    .execute(db)
    .await?;

    Ok(RemoteCommandApplyResult {
        command_id: command.command_id,
        appointment_id: appointment.id,
        status: "applied".to_string(),
        applied_at: Some(applied_at),
        message: Some(result_json),
    })
}

async fn apply_appointment_status_update(
    db: &SqlitePool,
    clinic_id: &str,
    actor_user_id: Option<&str>,
    input: UpdateAppointmentStatusInput,
    audit_action: &str,
    audit_metadata: serde_json::Value,
    emit_outbox: bool,
) -> AppResult<AppointmentSummary> {
    let status = input.status.trim().to_string();
    ensure_valid_appointment_status(&status)?;

    let previous_status: String = sqlx::query_scalar(
        "SELECT status FROM appointments WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL",
    )
    .bind(clinic_id)
    .bind(input.appointment_id.trim())
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::Validation("Cita no encontrada".to_string()))?;

    let now = now_utc();
    sqlx::query("UPDATE appointments SET status = ?, notes = COALESCE(?, notes), updated_at = ?, cancelled_at = CASE WHEN ? = 'cancelada' THEN ? ELSE cancelled_at END WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL")
        .bind(&status)
        .bind(input.notes.as_deref().map(str::trim))
        .bind(&now)
        .bind(&status)
        .bind(&now)
        .bind(clinic_id)
        .bind(input.appointment_id.trim())
        .execute(db)
        .await?;

    insert_event(
        db,
        clinic_id,
        input.appointment_id.trim(),
        actor_user_id,
        "status_changed",
        Some(&previous_status),
        Some(&status),
        input.notes.as_deref(),
    )
    .await?;
    log_action(
        db,
        Some(clinic_id),
        actor_user_id,
        audit_action,
        "appointments",
        Some(input.appointment_id.trim()),
        "info",
        Some(json!({
            "from": previous_status.clone(),
            "to": status.clone(),
            "context": audit_metadata
        })),
    )
    .await?;

    let appointment = get_appointment_by_id(db, clinic_id, input.appointment_id.trim()).await?;
    if emit_outbox {
        sync_service::enqueue_local_event(
            db,
            clinic_id,
            "appointments",
            input.appointment_id.trim(),
            "appointment.status_changed",
            json!({
                "appointment": appointment.clone(),
                "previousStatus": previous_status
            }),
        )
        .await?;
    }

    Ok(appointment)
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
        "UPDATE appointments SET status = 'cancelada', deleted_at = ?, deleted_by_user_id = ?, cancelled_at = COALESCE(cancelled_at, ?), updated_at = ? WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL",
    )
    .bind(&now)
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
        Some(&ctx.user_id),
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
    sync_service::enqueue_local_event(
        db,
        &ctx.clinic_id,
        "appointments",
        appointment_id.trim(),
        "appointment.deleted",
        json!({
            "appointment": appointment.clone(),
            "deletedAt": now
        }),
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
    user_id: Option<&str>,
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
