use serde_json::json;
use sqlx::SqlitePool;

use crate::errors::{AppError, AppResult};
use crate::models::{OdontogramEntry, OdontogramRecordView, UpsertOdontogramEntryInput};
use crate::services::audit_service::log_action;
use crate::services::auth_service::{validate_session, validate_session_for_intent};
use crate::services::guard::ensure_active_patient;
use crate::services::license_service::AccessIntent;
use crate::utils::{new_id, now_utc};

pub async fn get_odontogram(
    db: &SqlitePool,
    session_token: &str,
    patient_id: &str,
    dentition_type: &str,
) -> AppResult<OdontogramRecordView> {
    let ctx = validate_session(db, session_token, Some("odontogram.view")).await?;
    let patient_id = patient_id.trim().to_string();
    ensure_active_patient(db, &ctx.clinic_id, &patient_id).await?;
    let record_id = sqlx::query_scalar::<_, String>(
        r#"
        SELECT id FROM odontogram_records
        WHERE clinic_id = ? AND patient_id = ? AND dentition_type = ? AND status = 'active'
        LIMIT 1
        "#,
    )
    .bind(&ctx.clinic_id)
    .bind(&patient_id)
    .bind(dentition_type)
    .fetch_optional(db)
    .await?;

    let Some(record_id) = record_id else {
        return Ok(OdontogramRecordView {
            id: String::new(),
            patient_id,
            dentition_type: dentition_type.to_string(),
            entries: Vec::new(),
        });
    };

    let entries = sqlx::query_as::<_, OdontogramEntry>(
        r#"
        SELECT id, tooth_number, surface, state, finding, updated_at
        FROM odontogram_entries
        WHERE clinic_id = ? AND odontogram_record_id = ?
        ORDER BY tooth_number, surface
        "#,
    )
    .bind(&ctx.clinic_id)
    .bind(&record_id)
    .fetch_all(db)
    .await?;

    Ok(OdontogramRecordView {
        id: record_id,
        patient_id,
        dentition_type: dentition_type.to_string(),
        entries,
    })
}

pub async fn upsert_odontogram_entry(
    db: &SqlitePool,
    session_token: &str,
    input: UpsertOdontogramEntryInput,
) -> AppResult<OdontogramRecordView> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("odontogram.edit"),
        AccessIntent::DataWrite,
    )
    .await?;
    if input.patient_id.trim().is_empty()
        || input.tooth_number.trim().is_empty()
        || input.state.trim().is_empty()
    {
        return Err(AppError::Validation(
            "Paciente, pieza y estado son obligatorios".to_string(),
        ));
    }
    let patient_id = input.patient_id.trim().to_string();
    ensure_active_patient(db, &ctx.clinic_id, &patient_id).await?;

    let record_id = ensure_record(db, &ctx.clinic_id, &patient_id, &input.dentition_type).await?;
    let surface = input.surface.clone().unwrap_or_else(|| "all".to_string());
    let now = now_utc();

    let existing: Option<(String, String)> = sqlx::query_as(
        r#"
        SELECT id, state FROM odontogram_entries
        WHERE clinic_id = ? AND odontogram_record_id = ? AND tooth_number = ? AND surface = ?
        "#,
    )
    .bind(&ctx.clinic_id)
    .bind(&record_id)
    .bind(input.tooth_number.trim())
    .bind(&surface)
    .fetch_optional(db)
    .await?;

    let entry_id = if let Some((entry_id, previous_state)) = existing {
        sqlx::query(
            r#"
            UPDATE odontogram_entries
            SET state = ?, finding = ?, updated_by_user_id = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(input.state.trim())
        .bind(input.finding.as_deref().map(str::trim))
        .bind(&ctx.user_id)
        .bind(&now)
        .bind(&entry_id)
        .execute(db)
        .await?;

        insert_history(
            db,
            &ctx.clinic_id,
            Some(&entry_id),
            &record_id,
            &input.tooth_number,
            &surface,
            Some(&previous_state),
            &input.state,
            input.finding.as_deref(),
            &ctx.user_id,
        )
        .await?;
        entry_id
    } else {
        let entry_id = new_id();
        sqlx::query(
            r#"
            INSERT INTO odontogram_entries
              (id, clinic_id, odontogram_record_id, tooth_number, surface, state, finding,
               updated_by_user_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&entry_id)
        .bind(&ctx.clinic_id)
        .bind(&record_id)
        .bind(input.tooth_number.trim())
        .bind(&surface)
        .bind(input.state.trim())
        .bind(input.finding.as_deref().map(str::trim))
        .bind(&ctx.user_id)
        .bind(&now)
        .bind(&now)
        .execute(db)
        .await?;

        insert_history(
            db,
            &ctx.clinic_id,
            Some(&entry_id),
            &record_id,
            &input.tooth_number,
            &surface,
            None,
            &input.state,
            input.finding.as_deref(),
            &ctx.user_id,
        )
        .await?;
        entry_id
    };

    sqlx::query("UPDATE odontogram_records SET updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&record_id)
        .execute(db)
        .await?;

    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "odontogram.entry.upsert",
        "odontogram_entries",
        Some(&entry_id),
        "clinical",
        Some(json!({
            "patientId": &patient_id,
            "tooth": input.tooth_number,
            "surface": surface,
            "state": input.state
        })),
    )
    .await?;

    get_odontogram(db, session_token, &patient_id, &input.dentition_type).await
}

async fn ensure_record(
    db: &SqlitePool,
    clinic_id: &str,
    patient_id: &str,
    dentition_type: &str,
) -> AppResult<String> {
    if let Some(record_id) = sqlx::query_scalar::<_, String>(
        r#"
        SELECT id FROM odontogram_records
        WHERE clinic_id = ? AND patient_id = ? AND dentition_type = ? AND status = 'active'
        LIMIT 1
        "#,
    )
    .bind(clinic_id)
    .bind(patient_id)
    .bind(dentition_type)
    .fetch_optional(db)
    .await?
    {
        return Ok(record_id);
    }

    let record_id = new_id();
    let now = now_utc();
    sqlx::query(
        r#"
        INSERT INTO odontogram_records
          (id, clinic_id, patient_id, dentition_type, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, 'active', ?, ?)
        "#,
    )
    .bind(&record_id)
    .bind(clinic_id)
    .bind(patient_id)
    .bind(dentition_type)
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;

    Ok(record_id)
}

#[expect(
    clippy::too_many_arguments,
    reason = "Odontogram history rows preserve explicit clinical audit fields."
)]
async fn insert_history(
    db: &SqlitePool,
    clinic_id: &str,
    entry_id: Option<&str>,
    record_id: &str,
    tooth_number: &str,
    surface: &str,
    previous_state: Option<&str>,
    new_state: &str,
    finding: Option<&str>,
    user_id: &str,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO odontogram_entry_history
          (id, clinic_id, odontogram_entry_id, odontogram_record_id, tooth_number, surface,
           previous_state, new_state, finding, changed_by_user_id, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(new_id())
    .bind(clinic_id)
    .bind(entry_id)
    .bind(record_id)
    .bind(tooth_number.trim())
    .bind(surface)
    .bind(previous_state)
    .bind(new_state.trim())
    .bind(finding.map(str::trim))
    .bind(user_id)
    .bind(now_utc())
    .execute(db)
    .await?;

    Ok(())
}
