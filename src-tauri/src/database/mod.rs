use std::fs;
use std::path::PathBuf;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{ConnectOptions, SqlitePool};
use tauri::{AppHandle, Manager};

use crate::errors::AppResult;
use crate::services::backup_service;
use crate::utils::{new_id, now_utc};

pub(crate) static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub app_data_dir: PathBuf,
    pub files_dir: PathBuf,
    pub backups_dir: PathBuf,
    pub reports_dir: PathBuf,
}

pub async fn init(app: &AppHandle) -> AppResult<AppState> {
    let app_data_dir = app.path().app_data_dir()?;
    let data_dir = app_data_dir.join("data");
    let files_dir = app_data_dir.join("files");
    let backups_dir = app
        .path()
        .document_dir()
        .unwrap_or_else(|_| app_data_dir.clone())
        .join("DentalCare Backups");
    let reports_dir = app
        .path()
        .document_dir()
        .unwrap_or_else(|_| app_data_dir.clone())
        .join("DentalCare Reports");

    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(&files_dir)?;
    fs::create_dir_all(&backups_dir)?;
    fs::create_dir_all(&reports_dir)?;

    let restore_applied =
        backup_service::apply_pending_restore_if_needed(&app_data_dir, &data_dir, &files_dir)?;

    let db_path = data_dir.join("dentalcare.sqlite");
    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .log_statements(log::LevelFilter::Off);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA busy_timeout = 5000")
        .execute(&pool)
        .await?;
    MIGRATOR.run(&pool).await?;
    if let Some(metadata) = restore_applied {
        record_restore_applied(&pool, metadata).await?;
    }

    Ok(AppState {
        db: pool,
        app_data_dir,
        files_dir,
        backups_dir,
        reports_dir,
    })
}

async fn record_restore_applied(pool: &SqlitePool, metadata: serde_json::Value) -> AppResult<()> {
    let clinic_id: Option<String> =
        sqlx::query_scalar("SELECT id FROM clinics ORDER BY created_at LIMIT 1")
            .fetch_optional(pool)
            .await?;
    sqlx::query(
        r#"
        INSERT INTO audit_logs
          (id, clinic_id, user_id, action, entity_type, entity_id, severity, metadata, created_at)
        VALUES (?, ?, NULL, 'restore.applied', 'restore_jobs', NULL, 'security', ?, ?)
        "#,
    )
    .bind(new_id())
    .bind(clinic_id)
    .bind(metadata.to_string())
    .bind(now_utc())
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePoolOptions;

    use super::MIGRATOR;

    #[tokio::test]
    async fn migrations_create_core_tables() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect in-memory sqlite");

        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .expect("enable foreign keys");
        MIGRATOR.run(&pool).await.expect("run migrations");

        let permissions_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM permissions")
            .fetch_one(&pool)
            .await
            .expect("count permissions");
        let patients_table: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'patients'",
        )
        .fetch_one(&pool)
        .await
        .expect("patients table exists");
        let related_file_columns: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM pragma_table_info('files') WHERE name IN ('related_entity_type', 'related_entity_id')",
        )
        .fetch_one(&pool)
        .await
        .expect("payment proof columns exist");
        let alert_state_table: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'system_alert_state'",
        )
        .fetch_one(&pool)
        .await
        .expect("system alert state table exists");
        let report_export_columns: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM pragma_table_info('report_exports') WHERE name IN ('path', 'size_bytes')",
        )
        .fetch_one(&pool)
        .await
        .expect("report export columns exist");
        let license_table: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'app_license'",
        )
        .fetch_one(&pool)
        .await
        .expect("license table exists");

        assert!(permissions_count >= 10);
        assert_eq!(patients_table, 1);
        assert_eq!(related_file_columns, 2);
        assert_eq!(alert_state_table, 1);
        assert_eq!(report_export_columns, 2);
        assert_eq!(license_table, 1);
    }
}
