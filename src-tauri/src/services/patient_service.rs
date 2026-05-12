use serde_json::json;
use sqlx::SqlitePool;

use crate::errors::{AppError, AppResult};
use crate::models::{CreatePatientInput, ListPatientsInput, PatientSummary};
use crate::services::audit_service::log_action;
use crate::services::auth_service::validate_session;
use crate::utils::{new_id, normalize_search, now_utc};

pub async fn create_patient(
    db: &SqlitePool,
    session_token: &str,
    input: CreatePatientInput,
) -> AppResult<PatientSummary> {
    let ctx = validate_session(db, session_token, Some("patients.create")).await?;
    if input.full_name.trim().is_empty() {
        return Err(AppError::Validation(
            "El nombre completo del paciente es obligatorio".to_string(),
        ));
    }

    let id = new_id();
    let now = now_utc();

    sqlx::query(
        r#"
        INSERT INTO patients
          (id, clinic_id, full_name, birth_date, sex, phone, whatsapp, email, address,
           emergency_contact_name, emergency_contact_phone, occupation, allergies,
           systemic_diseases, current_medications, relevant_history, habits, general_notes,
           status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'active', ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&ctx.clinic_id)
    .bind(input.full_name.trim())
    .bind(
        input
            .birth_date
            .as_deref()
            .filter(|value| !value.is_empty()),
    )
    .bind(input.sex.as_deref().filter(|value| !value.is_empty()))
    .bind(input.phone.as_deref().map(str::trim))
    .bind(input.whatsapp.as_deref().map(str::trim))
    .bind(input.email.as_deref().map(str::trim))
    .bind(input.address.as_deref().map(str::trim))
    .bind(input.emergency_contact_name.as_deref().map(str::trim))
    .bind(input.emergency_contact_phone.as_deref().map(str::trim))
    .bind(input.occupation.as_deref().map(str::trim))
    .bind(input.allergies.as_deref().map(str::trim))
    .bind(input.systemic_diseases.as_deref().map(str::trim))
    .bind(input.current_medications.as_deref().map(str::trim))
    .bind(input.relevant_history.as_deref().map(str::trim))
    .bind(input.habits.as_deref().map(str::trim))
    .bind(input.general_notes.as_deref().map(str::trim))
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;

    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "patients.create",
        "patients",
        Some(&id),
        "info",
        Some(json!({ "fullName": input.full_name })),
    )
    .await?;

    get_patient_by_id(db, &ctx.clinic_id, &id).await
}

pub async fn list_patients(
    db: &SqlitePool,
    session_token: &str,
    input: ListPatientsInput,
) -> AppResult<Vec<PatientSummary>> {
    let ctx = validate_session(db, session_token, Some("patients.view")).await?;
    let limit = input.limit.unwrap_or(50).clamp(1, 200);
    let search = normalize_search(input.search.as_deref().unwrap_or(""));

    let patients = sqlx::query_as::<_, PatientSummary>(
        r#"
        SELECT id, full_name, birth_date, sex, phone, whatsapp, email, allergies,
               systemic_diseases, current_medications, status, created_at, updated_at
        FROM patients
        WHERE clinic_id = ?
          AND deleted_at IS NULL
          AND (? = '%%'
            OR full_name LIKE ?
            OR IFNULL(phone, '') LIKE ?
            OR IFNULL(whatsapp, '') LIKE ?
            OR IFNULL(email, '') LIKE ?)
        ORDER BY updated_at DESC
        LIMIT ?
        "#,
    )
    .bind(ctx.clinic_id)
    .bind(&search)
    .bind(&search)
    .bind(&search)
    .bind(&search)
    .bind(&search)
    .bind(limit)
    .fetch_all(db)
    .await?;

    Ok(patients)
}

pub async fn get_patient(
    db: &SqlitePool,
    session_token: &str,
    patient_id: &str,
) -> AppResult<PatientSummary> {
    let ctx = validate_session(db, session_token, Some("patients.view")).await?;
    get_patient_by_id(db, &ctx.clinic_id, patient_id).await
}

async fn get_patient_by_id(
    db: &SqlitePool,
    clinic_id: &str,
    patient_id: &str,
) -> AppResult<PatientSummary> {
    sqlx::query_as::<_, PatientSummary>(
        r#"
        SELECT id, full_name, birth_date, sex, phone, whatsapp, email, allergies,
               systemic_diseases, current_medications, status, created_at, updated_at
        FROM patients
        WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL
        "#,
    )
    .bind(clinic_id)
    .bind(patient_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::Validation("Paciente no encontrado".to_string()))
}
