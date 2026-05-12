use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use chrono::Local;
use serde_json::json;
use sqlx::SqlitePool;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

use crate::database::AppState;
use crate::errors::AppResult;
use crate::models::BackupResult;
use crate::services::audit_service::log_action;
use crate::services::auth_service::validate_session;
use crate::utils::{new_id, now_utc};

pub async fn create_backup(state: &AppState, session_token: &str) -> AppResult<BackupResult> {
    let ctx = validate_session(&state.db, session_token, Some("backups.create")).await?;
    fs::create_dir_all(&state.backups_dir)?;

    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let backup_id = new_id();
    let temp_db_path = state
        .backups_dir
        .join(format!("dentalcare-{timestamp}.sqlite"));
    let backup_path = state
        .backups_dir
        .join(format!("dentalcare-backup-{timestamp}.zip"));

    if temp_db_path.exists() {
        fs::remove_file(&temp_db_path)?;
    }
    if backup_path.exists() {
        fs::remove_file(&backup_path)?;
    }

    let temp_db = temp_db_path.to_string_lossy().to_string();
    sqlx::query("VACUUM INTO ?")
        .bind(&temp_db)
        .execute(&state.db)
        .await?;

    create_zip_archive(
        &backup_path,
        &temp_db_path,
        &state.files_dir,
        &backup_id,
        &ctx.clinic_id,
    )?;
    let _ = fs::remove_file(&temp_db_path);

    let size_bytes = fs::metadata(&backup_path)?.len();
    let created_at = now_utc();
    let manifest = json!({
        "id": backup_id,
        "product": "DentalCare Manager",
        "version": env!("CARGO_PKG_VERSION"),
        "clinicId": ctx.clinic_id,
        "createdAt": created_at,
        "includes": ["database", "files"]
    });

    sqlx::query(
        r#"
        INSERT INTO backups
          (id, clinic_id, created_by_user_id, path, status, size_bytes, manifest, created_at)
        VALUES (?, ?, ?, ?, 'completed', ?, ?, ?)
        "#,
    )
    .bind(&backup_id)
    .bind(&ctx.clinic_id)
    .bind(&ctx.user_id)
    .bind(backup_path.to_string_lossy().to_string())
    .bind(size_bytes as i64)
    .bind(manifest.to_string())
    .bind(&created_at)
    .execute(&state.db)
    .await?;

    log_action(
        &state.db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "backups.create",
        "backups",
        Some(&backup_id),
        "security",
        Some(json!({ "path": backup_path.to_string_lossy(), "sizeBytes": size_bytes })),
    )
    .await?;

    Ok(BackupResult {
        id: backup_id,
        path: backup_path.to_string_lossy().to_string(),
        size_bytes,
        created_at,
    })
}

fn create_zip_archive(
    backup_path: &Path,
    db_path: &Path,
    files_dir: &Path,
    backup_id: &str,
    clinic_id: &str,
) -> AppResult<()> {
    let file = File::create(backup_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    let manifest = json!({
        "id": backup_id,
        "clinicId": clinic_id,
        "createdAt": now_utc(),
        "database": "data/dentalcare.sqlite",
        "filesRoot": "files/"
    });

    zip.start_file("manifest.json", options)?;
    zip.write_all(manifest.to_string().as_bytes())?;

    zip.start_file("data/dentalcare.sqlite", options)?;
    let mut db_file = File::open(db_path)?;
    let mut buffer = Vec::new();
    db_file.read_to_end(&mut buffer)?;
    zip.write_all(&buffer)?;

    if files_dir.exists() {
        for entry in WalkDir::new(files_dir).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                let relative = entry
                    .path()
                    .strip_prefix(files_dir)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .replace('\\', "/");
                zip.start_file(format!("files/{relative}"), options)?;
                let mut source = File::open(entry.path())?;
                let mut file_buffer = Vec::new();
                source.read_to_end(&mut file_buffer)?;
                zip.write_all(&file_buffer)?;
            }
        }
    }

    zip.finish()?;
    Ok(())
}

#[allow(dead_code)]
async fn _touch_db(_db: &SqlitePool) {}
