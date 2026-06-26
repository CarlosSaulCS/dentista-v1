use sqlx::SqlitePool;

use crate::errors::{AppError, AppResult};

pub async fn ensure_active_patient(
    db: &SqlitePool,
    clinic_id: &str,
    patient_id: &str,
) -> AppResult<()> {
    let patient_id = patient_id.trim();
    if patient_id.is_empty() {
        return Err(AppError::Validation("Selecciona un paciente".to_string()));
    }

    let exists: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM patients
        WHERE clinic_id = ?
          AND id = ?
          AND deleted_at IS NULL
        "#,
    )
    .bind(clinic_id)
    .bind(patient_id)
    .fetch_one(db)
    .await?;

    if exists == 0 {
        return Err(AppError::Validation(
            "Paciente no encontrado o dado de baja".to_string(),
        ));
    }

    Ok(())
}

pub async fn ensure_optional_active_patient(
    db: &SqlitePool,
    clinic_id: &str,
    patient_id: Option<&str>,
) -> AppResult<()> {
    if let Some(patient_id) = patient_id.map(str::trim).filter(|value| !value.is_empty()) {
        ensure_active_patient(db, clinic_id, patient_id).await?;
    }

    Ok(())
}
