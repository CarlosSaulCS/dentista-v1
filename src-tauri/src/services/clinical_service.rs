use serde_json::json;
use sqlx::SqlitePool;

use crate::errors::{AppError, AppResult};
use crate::models::{
    ClinicalEvolutionSummary, ClinicalRecordSummary, CreateClinicalEvolutionInput,
    CreateClinicalRecordInput,
};
use crate::services::audit_service::log_action;
use crate::services::auth_service::validate_session;
use crate::utils::{new_id, now_utc};

pub async fn create_clinical_record(
    db: &SqlitePool,
    session_token: &str,
    input: CreateClinicalRecordInput,
) -> AppResult<ClinicalRecordSummary> {
    let ctx = validate_session(db, session_token, Some("clinical.edit")).await?;
    if input.patient_id.trim().is_empty() {
        return Err(AppError::Validation(
            "El paciente es obligatorio".to_string(),
        ));
    }

    let id = new_id();
    let now = now_utc();

    sqlx::query(
        r#"
        INSERT INTO clinical_records
          (id, clinic_id, patient_id, responsible_user_id, appointment_id, chief_complaint,
           current_condition, hereditary_history, pathological_history, non_pathological_history,
           allergies, current_medications, systemic_diseases, habits, clinical_exploration,
           diagnosis, prognosis, suggested_plan, indications, observations, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&ctx.clinic_id)
    .bind(&input.patient_id)
    .bind(&ctx.user_id)
    .bind(input.appointment_id.as_deref())
    .bind(input.chief_complaint.as_deref().map(str::trim))
    .bind(input.current_condition.as_deref().map(str::trim))
    .bind(input.hereditary_history.as_deref().map(str::trim))
    .bind(input.pathological_history.as_deref().map(str::trim))
    .bind(input.non_pathological_history.as_deref().map(str::trim))
    .bind(input.allergies.as_deref().map(str::trim))
    .bind(input.current_medications.as_deref().map(str::trim))
    .bind(input.systemic_diseases.as_deref().map(str::trim))
    .bind(input.habits.as_deref().map(str::trim))
    .bind(input.clinical_exploration.as_deref().map(str::trim))
    .bind(input.diagnosis.as_deref().map(str::trim))
    .bind(input.prognosis.as_deref().map(str::trim))
    .bind(input.suggested_plan.as_deref().map(str::trim))
    .bind(input.indications.as_deref().map(str::trim))
    .bind(input.observations.as_deref().map(str::trim))
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;

    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "clinical.record.create",
        "clinical_records",
        Some(&id),
        "clinical",
        Some(json!({ "patientId": input.patient_id })),
    )
    .await?;

    get_clinical_record_by_id(db, &ctx.clinic_id, &id).await
}

pub async fn create_clinical_evolution(
    db: &SqlitePool,
    session_token: &str,
    input: CreateClinicalEvolutionInput,
) -> AppResult<ClinicalEvolutionSummary> {
    let ctx = validate_session(db, session_token, Some("clinical.edit")).await?;
    if input.patient_id.trim().is_empty() || input.reason.trim().is_empty() {
        return Err(AppError::Validation(
            "Paciente y motivo son obligatorios".to_string(),
        ));
    }

    let id = new_id();
    let now = now_utc();

    sqlx::query(
        r#"
        INSERT INTO clinical_evolutions
          (id, clinic_id, patient_id, clinical_record_id, appointment_id, responsible_user_id,
           reason, findings, procedures_done, indications, next_appointment_notes, signed_by,
           created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&ctx.clinic_id)
    .bind(&input.patient_id)
    .bind(input.clinical_record_id.as_deref())
    .bind(input.appointment_id.as_deref())
    .bind(&ctx.user_id)
    .bind(input.reason.trim())
    .bind(input.findings.as_deref().map(str::trim))
    .bind(input.procedures_done.as_deref().map(str::trim))
    .bind(input.indications.as_deref().map(str::trim))
    .bind(input.next_appointment_notes.as_deref().map(str::trim))
    .bind(&ctx.full_name)
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;

    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "clinical.evolution.create",
        "clinical_evolutions",
        Some(&id),
        "clinical",
        Some(json!({ "patientId": input.patient_id })),
    )
    .await?;

    get_clinical_evolution_by_id(db, &ctx.clinic_id, &id).await
}

pub async fn list_clinical_records(
    db: &SqlitePool,
    session_token: &str,
    patient_id: &str,
) -> AppResult<Vec<ClinicalRecordSummary>> {
    let ctx = validate_session(db, session_token, Some("clinical.view")).await?;
    let rows = sqlx::query_as::<_, ClinicalRecordSummary>(
        r#"
        SELECT cr.id, cr.patient_id, p.full_name AS patient_name, cr.responsible_user_id,
               u.full_name AS responsible_name, cr.chief_complaint, cr.diagnosis,
               cr.suggested_plan, cr.created_at, cr.updated_at
        FROM clinical_records cr
        JOIN patients p ON p.id = cr.patient_id
        LEFT JOIN users u ON u.id = cr.responsible_user_id
        WHERE cr.clinic_id = ? AND cr.patient_id = ?
        ORDER BY cr.created_at DESC
        "#,
    )
    .bind(ctx.clinic_id)
    .bind(patient_id)
    .fetch_all(db)
    .await?;

    Ok(rows)
}

pub async fn list_clinical_evolutions(
    db: &SqlitePool,
    session_token: &str,
    patient_id: &str,
) -> AppResult<Vec<ClinicalEvolutionSummary>> {
    let ctx = validate_session(db, session_token, Some("clinical.view")).await?;
    let rows = sqlx::query_as::<_, ClinicalEvolutionSummary>(
        r#"
        SELECT ce.id, ce.patient_id, p.full_name AS patient_name, u.full_name AS responsible_name,
               ce.reason, ce.findings, ce.procedures_done, ce.indications,
               ce.next_appointment_notes, ce.created_at
        FROM clinical_evolutions ce
        JOIN patients p ON p.id = ce.patient_id
        JOIN users u ON u.id = ce.responsible_user_id
        WHERE ce.clinic_id = ? AND ce.patient_id = ? AND ce.voided_at IS NULL
        ORDER BY ce.created_at DESC
        "#,
    )
    .bind(ctx.clinic_id)
    .bind(patient_id)
    .fetch_all(db)
    .await?;

    Ok(rows)
}

async fn get_clinical_record_by_id(
    db: &SqlitePool,
    clinic_id: &str,
    record_id: &str,
) -> AppResult<ClinicalRecordSummary> {
    sqlx::query_as::<_, ClinicalRecordSummary>(
        r#"
        SELECT cr.id, cr.patient_id, p.full_name AS patient_name, cr.responsible_user_id,
               u.full_name AS responsible_name, cr.chief_complaint, cr.diagnosis,
               cr.suggested_plan, cr.created_at, cr.updated_at
        FROM clinical_records cr
        JOIN patients p ON p.id = cr.patient_id
        LEFT JOIN users u ON u.id = cr.responsible_user_id
        WHERE cr.clinic_id = ? AND cr.id = ?
        "#,
    )
    .bind(clinic_id)
    .bind(record_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::Validation("Historia clínica no encontrada".to_string()))
}

async fn get_clinical_evolution_by_id(
    db: &SqlitePool,
    clinic_id: &str,
    evolution_id: &str,
) -> AppResult<ClinicalEvolutionSummary> {
    sqlx::query_as::<_, ClinicalEvolutionSummary>(
        r#"
        SELECT ce.id, ce.patient_id, p.full_name AS patient_name, u.full_name AS responsible_name,
               ce.reason, ce.findings, ce.procedures_done, ce.indications,
               ce.next_appointment_notes, ce.created_at
        FROM clinical_evolutions ce
        JOIN patients p ON p.id = ce.patient_id
        JOIN users u ON u.id = ce.responsible_user_id
        WHERE ce.clinic_id = ? AND ce.id = ?
        "#,
    )
    .bind(clinic_id)
    .bind(evolution_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::Validation("Evolución clínica no encontrada".to_string()))
}
