use serde_json::json;
use sqlx::SqlitePool;

use crate::errors::{AppError, AppResult};
use crate::models::{CreatePatientInput, ListPatientsInput, PatientSummary, UpdatePatientInput};
use crate::services::audit_service::log_action;
use crate::services::auth_service::{validate_session, validate_session_for_intent};
use crate::services::license_service::AccessIntent;
use crate::utils::{new_id, normalize_search, now_utc};

pub async fn create_patient(
    db: &SqlitePool,
    session_token: &str,
    input: CreatePatientInput,
) -> AppResult<PatientSummary> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("patients.create"),
        AccessIntent::DataWrite,
    )
    .await?;
    if input.full_name.trim().is_empty() {
        return Err(AppError::Validation(
            "El nombre completo del paciente es obligatorio".to_string(),
        ));
    }
    ensure_no_duplicate_patient(
        db,
        &ctx.clinic_id,
        &input.full_name,
        input.phone.as_deref(),
        input.email.as_deref(),
        None,
    )
    .await?;

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
        SELECT id, full_name, birth_date, sex, phone, whatsapp, email, address,
               emergency_contact_name, emergency_contact_phone, occupation, allergies,
               systemic_diseases, current_medications, relevant_history, habits,
               general_notes, status, created_at, updated_at
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

pub async fn update_patient(
    db: &SqlitePool,
    session_token: &str,
    input: UpdatePatientInput,
) -> AppResult<PatientSummary> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("patients.edit"),
        AccessIntent::DataWrite,
    )
    .await?;
    if input.id.trim().is_empty() {
        return Err(AppError::Validation(
            "Selecciona un paciente para editar".to_string(),
        ));
    }
    if input.full_name.trim().is_empty() {
        return Err(AppError::Validation(
            "El nombre completo del paciente es obligatorio".to_string(),
        ));
    }
    ensure_no_duplicate_patient(
        db,
        &ctx.clinic_id,
        &input.full_name,
        input.phone.as_deref(),
        input.email.as_deref(),
        Some(input.id.trim()),
    )
    .await?;

    let now = now_utc();
    let result = sqlx::query(
        r#"
        UPDATE patients
        SET full_name = ?, birth_date = ?, sex = ?, phone = ?, whatsapp = ?, email = ?,
            address = ?, emergency_contact_name = ?, emergency_contact_phone = ?,
            occupation = ?, allergies = ?, systemic_diseases = ?, current_medications = ?,
            relevant_history = ?, habits = ?, general_notes = ?, status = ?, updated_at = ?
        WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL
        "#,
    )
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
    .bind(input.status.trim())
    .bind(&now)
    .bind(&ctx.clinic_id)
    .bind(input.id.trim())
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Validation("Paciente no encontrado".to_string()));
    }

    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "patients.update",
        "patients",
        Some(input.id.trim()),
        "info",
        Some(json!({ "fullName": input.full_name, "status": input.status })),
    )
    .await?;

    get_patient_by_id(db, &ctx.clinic_id, input.id.trim()).await
}

async fn ensure_no_duplicate_patient(
    db: &SqlitePool,
    clinic_id: &str,
    full_name: &str,
    phone: Option<&str>,
    email: Option<&str>,
    except_id: Option<&str>,
) -> AppResult<()> {
    let normalized_name = full_name.trim().to_lowercase();
    let phone = phone.map(str::trim).filter(|value| !value.is_empty());
    let email = email.map(str::trim).filter(|value| !value.is_empty());
    if phone.is_none() && email.is_none() {
        return Ok(());
    }
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM patients
        WHERE clinic_id = ?
          AND deleted_at IS NULL
          AND lower(full_name) = ?
          AND (? IS NULL OR id <> ?)
          AND ((? IS NOT NULL AND phone = ?) OR (? IS NOT NULL AND email = ?))
        "#,
    )
    .bind(clinic_id)
    .bind(normalized_name)
    .bind(except_id)
    .bind(except_id)
    .bind(phone)
    .bind(phone)
    .bind(email)
    .bind(email)
    .fetch_one(db)
    .await?;
    if count > 0 {
        return Err(AppError::Validation(
            "Ya existe un paciente activo con el mismo nombre y contacto".to_string(),
        ));
    }
    Ok(())
}

pub async fn soft_delete_patient(
    db: &SqlitePool,
    session_token: &str,
    patient_id: &str,
) -> AppResult<PatientSummary> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("patients.delete"),
        AccessIntent::DataWrite,
    )
    .await?;
    if patient_id.trim().is_empty() {
        return Err(AppError::Validation(
            "Selecciona un paciente para dar de baja".to_string(),
        ));
    }

    let patient = get_patient_by_id(db, &ctx.clinic_id, patient_id).await?;
    let now = now_utc();
    sqlx::query(
        r#"
        UPDATE patients
        SET status = 'inactive', deleted_at = ?, deleted_by_user_id = ?, updated_at = ?
        WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL
        "#,
    )
    .bind(&now)
    .bind(&ctx.user_id)
    .bind(&now)
    .bind(&ctx.clinic_id)
    .bind(patient_id.trim())
    .execute(db)
    .await?;

    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "patients.soft_delete",
        "patients",
        Some(patient_id.trim()),
        "warning",
        Some(json!({ "fullName": patient.full_name })),
    )
    .await?;

    Ok(patient)
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
        SELECT id, full_name, birth_date, sex, phone, whatsapp, email, address,
               emergency_contact_name, emergency_contact_phone, occupation, allergies,
               systemic_diseases, current_medications, relevant_history, habits,
               general_notes, status, created_at, updated_at
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
