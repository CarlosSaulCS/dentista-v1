use serde_json::Value;
use sqlx::SqlitePool;

use crate::errors::AppResult;
use crate::utils::{new_id, now_utc};

#[expect(
    clippy::too_many_arguments,
    reason = "Audit rows mirror the persisted audit log schema."
)]
pub async fn log_action(
    db: &SqlitePool,
    clinic_id: Option<&str>,
    user_id: Option<&str>,
    action: &str,
    entity_type: &str,
    entity_id: Option<&str>,
    severity: &str,
    metadata: Option<Value>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO audit_logs
          (id, clinic_id, user_id, action, entity_type, entity_id, severity, metadata, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(new_id())
    .bind(clinic_id)
    .bind(user_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(severity)
    .bind(metadata.map(|value| value.to_string()))
    .bind(now_utc())
    .execute(db)
    .await?;

    Ok(())
}
