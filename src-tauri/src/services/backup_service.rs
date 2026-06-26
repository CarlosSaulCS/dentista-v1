use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive};

use crate::database::AppState;
use crate::errors::{AppError, AppResult};
use crate::models::{
    BackupManifest, BackupResult, BackupSettings, BackupSummary, BackupVerificationResult,
    RestorePrepareResult, RestorePreview, UpdateBackupSettingsInput,
};
use crate::services::audit_service::log_action;
use crate::services::auth_service::{validate_session, validate_session_for_intent, AuthContext};
use crate::services::license_service::AccessIntent;
use crate::utils::{new_id, now_utc};

const BACKUP_DB_PATH: &str = "data/dentalcare.sqlite";
const CHECKSUMS_PATH: &str = "checksums.sha256";
const MANIFEST_PATH: &str = "manifest.json";
const SYSTEM_INFO_PATH: &str = "metadata/system-info.json";
const MAX_CLINICAL_FILE_BYTES: usize = 100 * 1024 * 1024;

struct BackupEntry {
    zip_path: String,
    bytes: Vec<u8>,
    checksum_sha256: String,
}

pub async fn create_backup(state: &AppState, session_token: &str) -> AppResult<BackupResult> {
    let ctx = validate_session_for_intent(
        &state.db,
        session_token,
        Some("backups.create"),
        AccessIntent::ExportOrBackup,
    )
    .await?;
    let settings = backup_settings_for_clinic(state, &ctx.clinic_id).await?;
    create_backup_for_context(state, &ctx, "manual", settings.include_files).await
}

pub async fn create_automatic_backup_if_due(
    state: &AppState,
    session_token: &str,
) -> AppResult<Option<BackupResult>> {
    let ctx = validate_session(&state.db, session_token, None).await?;
    let settings = backup_settings_for_clinic(state, &ctx.clinic_id).await?;
    if !settings.automatic_enabled {
        return Ok(None);
    }

    let last_backup: Option<String> = sqlx::query_scalar(
        "SELECT MAX(created_at) FROM backups WHERE clinic_id = ? AND status = 'completed'",
    )
    .bind(&ctx.clinic_id)
    .fetch_optional(&state.db)
    .await?;
    let stale = last_backup
        .as_deref()
        .and_then(parse_datetime)
        .map(|last| Utc::now().signed_duration_since(last).num_hours() >= 24)
        .unwrap_or(true);
    if !stale {
        return Ok(None);
    }

    create_backup_for_context(state, &ctx, "automatic", settings.include_files)
        .await
        .map(Some)
}

pub async fn list_backups(state: &AppState, session_token: &str) -> AppResult<Vec<BackupSummary>> {
    let ctx = validate_session(&state.db, session_token, Some("backups.create")).await?;
    let rows = sqlx::query_as::<_, BackupSummary>(
        r#"
        SELECT id, path, status, backup_type, size_bytes, checksum_sha256, verification_status,
               verified_at, app_version, migration_version, file_count, created_at
        FROM backups
        WHERE clinic_id = ?
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .bind(ctx.clinic_id)
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}

pub async fn get_backup_settings(
    state: &AppState,
    session_token: &str,
) -> AppResult<BackupSettings> {
    let ctx = validate_session(&state.db, session_token, Some("backups.create")).await?;
    backup_settings_for_clinic(state, &ctx.clinic_id).await
}

pub async fn update_backup_settings(
    state: &AppState,
    session_token: &str,
    input: UpdateBackupSettingsInput,
) -> AppResult<BackupSettings> {
    let ctx = validate_session_for_intent(
        &state.db,
        session_token,
        Some("backups.create"),
        AccessIntent::DataWrite,
    )
    .await?;
    if !matches!(input.frequency.as_str(), "daily" | "weekly" | "manual") {
        return Err(AppError::Validation(
            "Frecuencia de respaldo no soportada".to_string(),
        ));
    }
    let retention_limit = input.retention_limit.clamp(1, 365);
    let now = now_utc();
    sqlx::query(
        r#"
        INSERT INTO backup_settings
          (clinic_id, automatic_enabled, frequency, include_files, encrypt_backups,
           retention_limit, updated_by_user_id, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(clinic_id) DO UPDATE SET
          automatic_enabled = excluded.automatic_enabled,
          frequency = excluded.frequency,
          include_files = excluded.include_files,
          encrypt_backups = excluded.encrypt_backups,
          retention_limit = excluded.retention_limit,
          updated_by_user_id = excluded.updated_by_user_id,
          updated_at = excluded.updated_at
        "#,
    )
    .bind(&ctx.clinic_id)
    .bind(if input.automatic_enabled { 1 } else { 0 })
    .bind(input.frequency)
    .bind(if input.include_files { 1 } else { 0 })
    .bind(if input.encrypt_backups { 1 } else { 0 })
    .bind(retention_limit)
    .bind(&ctx.user_id)
    .bind(&now)
    .execute(&state.db)
    .await?;

    log_action(
        &state.db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "backups.settings_update",
        "backup_settings",
        Some(&ctx.clinic_id),
        "security",
        Some(json!({ "automaticEnabled": input.automatic_enabled, "retentionLimit": retention_limit })),
    )
    .await?;

    backup_settings_for_clinic(state, &ctx.clinic_id).await
}

pub async fn verify_backup(
    state: &AppState,
    session_token: &str,
    path: &str,
) -> AppResult<BackupVerificationResult> {
    let ctx = validate_session(&state.db, session_token, Some("backups.create")).await?;
    let result = verify_backup_path(Path::new(path))?;
    let status = if result.valid { "valid" } else { "invalid" };
    sqlx::query(
        "UPDATE backups SET verification_status = ?, verified_at = ? WHERE clinic_id = ? AND path = ?",
    )
    .bind(status)
    .bind(now_utc())
    .bind(&ctx.clinic_id)
    .bind(path)
    .execute(&state.db)
    .await?;
    Ok(result)
}

pub async fn preview_restore(
    state: &AppState,
    session_token: &str,
    path: &str,
) -> AppResult<RestorePreview> {
    validate_session(&state.db, session_token, Some("backups.restore")).await?;
    let verification = verify_backup_path(Path::new(path))?;
    let summary = verification
        .manifest
        .as_ref()
        .map(|manifest| {
            json!({
                "backupId": manifest.backup_id,
                "createdAt": manifest.created_at,
                "appVersion": manifest.app_version,
                "migrationVersion": manifest.migration_version,
                "includes": manifest.includes,
                "tableCounts": manifest.table_counts,
                "fileCount": manifest.file_count,
                "encrypted": manifest.encrypted
            })
        })
        .unwrap_or_else(|| json!({}));
    Ok(RestorePreview {
        valid: verification.valid,
        source_path: path.to_string(),
        manifest: verification.manifest,
        summary,
        errors: verification.errors,
    })
}

pub async fn prepare_restore(
    state: &AppState,
    session_token: &str,
    path: &str,
) -> AppResult<RestorePrepareResult> {
    let ctx = validate_session_for_intent(
        &state.db,
        session_token,
        Some("backups.restore"),
        AccessIntent::Restore,
    )
    .await?;
    let verification = verify_backup_path(Path::new(path))?;
    if !verification.valid {
        return Err(AppError::Validation(format!(
            "El respaldo no es válido: {}",
            verification.errors.join("; ")
        )));
    }

    let safety_backup = create_backup_for_context(state, &ctx, "pre_restore", true).await?;
    let restore_job_id = new_id();
    let pending_dir = state.app_data_dir.join("restore-pending");
    fs::create_dir_all(&pending_dir)?;
    let staged_path = pending_dir.join("restore.zip");
    fs::copy(path, &staged_path)?;
    let marker = json!({
        "restoreJobId": restore_job_id,
        "sourcePath": path,
        "stagedPath": staged_path.to_string_lossy(),
        "safetyBackupPath": safety_backup.path,
        "preparedByUserId": ctx.user_id,
        "clinicId": ctx.clinic_id,
        "preparedAt": now_utc()
    });
    fs::write(
        pending_dir.join("restore-info.json"),
        serde_json::to_vec_pretty(&marker)?,
    )?;

    sqlx::query(
        r#"
        INSERT INTO restore_jobs
          (id, clinic_id, backup_id, source_path, staged_path, safety_backup_path, status,
           manifest, verification_json, prepared_by_user_id, prepared_at)
        VALUES (?, ?, ?, ?, ?, ?, 'pending_restart', ?, ?, ?, ?)
        "#,
    )
    .bind(&restore_job_id)
    .bind(&ctx.clinic_id)
    .bind(verification.backup_id.as_deref())
    .bind(path)
    .bind(staged_path.to_string_lossy().to_string())
    .bind(&safety_backup.path)
    .bind(
        verification
            .manifest
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?,
    )
    .bind(serde_json::to_string(&verification)?)
    .bind(&ctx.user_id)
    .bind(now_utc())
    .execute(&state.db)
    .await?;

    log_action(
        &state.db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "restore.prepare",
        "restore_jobs",
        Some(&restore_job_id),
        "security",
        Some(json!({ "sourcePath": path, "safetyBackupPath": safety_backup.path })),
    )
    .await?;

    Ok(RestorePrepareResult {
        restore_job_id,
        staged_path: staged_path.to_string_lossy().to_string(),
        safety_backup_path: safety_backup.path,
        message:
            "Restauración preparada. Reinicia la aplicación para aplicarla de forma controlada."
                .to_string(),
    })
}

pub fn apply_pending_restore_if_needed(
    app_data_dir: &Path,
    data_dir: &Path,
    files_dir: &Path,
) -> AppResult<Option<Value>> {
    let pending_dir = app_data_dir.join("restore-pending");
    let marker_path = pending_dir.join("restore-info.json");
    let staged_path = pending_dir.join("restore.zip");
    if !marker_path.exists() || !staged_path.exists() {
        return Ok(None);
    }

    let marker: Value = serde_json::from_slice(&fs::read(&marker_path)?)?;
    let verification = verify_backup_path(&staged_path)?;
    if !verification.valid {
        let failed = json!({
            "status": "failed_verification",
            "errors": verification.errors,
            "failedAt": now_utc(),
            "marker": marker
        });
        fs::write(
            app_data_dir.join("restore-failed.json"),
            serde_json::to_vec_pretty(&failed)?,
        )?;
        let _ = fs::remove_file(&marker_path);
        return Ok(Some(failed));
    }

    fs::create_dir_all(data_dir)?;
    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let local_safety_dir = app_data_dir.join(format!("restore-local-safety-{timestamp}"));
    fs::create_dir_all(&local_safety_dir)?;
    preserve_existing_database(data_dir, &local_safety_dir)?;
    preserve_existing_files(files_dir, &local_safety_dir)?;

    let temp_db = data_dir.join("dentalcare.sqlite.restore");
    extract_zip_file(&staged_path, BACKUP_DB_PATH, &temp_db)?;
    let db_path = data_dir.join("dentalcare.sqlite");
    remove_sqlite_files(&db_path)?;
    fs::rename(&temp_db, &db_path)?;

    if zip_contains_prefix(&staged_path, "files/")? {
        if files_dir.exists() {
            fs::remove_dir_all(files_dir)?;
        }
        fs::create_dir_all(files_dir)?;
        extract_zip_prefix(&staged_path, "files/", files_dir)?;
    }

    let applied = json!({
        "status": "applied",
        "appliedAt": now_utc(),
        "localSafetyDir": local_safety_dir.to_string_lossy(),
        "manifest": verification.manifest,
        "marker": marker
    });
    fs::write(
        app_data_dir.join("restore-applied.json"),
        serde_json::to_vec_pretty(&applied)?,
    )?;
    let _ = fs::remove_dir_all(&pending_dir);
    Ok(Some(applied))
}

async fn create_backup_for_context(
    state: &AppState,
    ctx: &AuthContext,
    backup_type: &str,
    include_files: bool,
) -> AppResult<BackupResult> {
    fs::create_dir_all(&state.backups_dir)?;
    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let backup_id = new_id();
    let temp_db_path = state
        .backups_dir
        .join(format!("dentalcare-{backup_type}-{timestamp}.sqlite"));
    let backup_path = state
        .backups_dir
        .join(format!("dentalcare-{backup_type}-{timestamp}.zip"));

    if temp_db_path.exists() {
        fs::remove_file(&temp_db_path)?;
    }
    let temp_db = temp_db_path.to_string_lossy().to_string();
    sqlx::query("VACUUM INTO ?")
        .bind(&temp_db)
        .execute(&state.db)
        .await?;

    let table_counts = collect_table_counts(&state.db, &ctx.clinic_id).await?;
    let migration_version = migration_version(&state.db).await?;
    let manifest = create_zip_archive(
        &backup_path,
        &temp_db_path,
        &state.files_dir,
        &backup_id,
        &ctx.clinic_id,
        &ctx.user_id,
        include_files,
        table_counts.clone(),
        migration_version,
    )?;
    let _ = fs::remove_file(&temp_db_path);

    let size_bytes = fs::metadata(&backup_path)?.len();
    let archive_checksum = sha256_file(&backup_path)?;
    let created_at = manifest.created_at.clone();

    sqlx::query(
        r#"
        INSERT INTO backups
          (id, clinic_id, created_by_user_id, path, status, size_bytes, manifest, created_at,
           checksum_sha256, verification_status, backup_type, app_version, migration_version,
           file_count, table_counts_json)
        VALUES (?, ?, ?, ?, 'completed', ?, ?, ?, ?, 'not_verified', ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&backup_id)
    .bind(&ctx.clinic_id)
    .bind(&ctx.user_id)
    .bind(backup_path.to_string_lossy().to_string())
    .bind(size_bytes as i64)
    .bind(serde_json::to_string(&manifest)?)
    .bind(&created_at)
    .bind(&archive_checksum)
    .bind(backup_type)
    .bind(env!("CARGO_PKG_VERSION"))
    .bind(migration_version)
    .bind(manifest.file_count)
    .bind(table_counts.to_string())
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
        Some(json!({
            "path": backup_path.to_string_lossy(),
            "sizeBytes": size_bytes,
            "backupType": backup_type,
            "checksumSha256": archive_checksum
        })),
    )
    .await?;

    Ok(BackupResult {
        id: backup_id,
        path: backup_path.to_string_lossy().to_string(),
        size_bytes,
        created_at,
        checksum_sha256: Some(archive_checksum),
    })
}

fn create_zip_archive(
    backup_path: &Path,
    db_path: &Path,
    files_dir: &Path,
    backup_id: &str,
    clinic_id: &str,
    created_by_user_id: &str,
    include_files: bool,
    table_counts: Value,
    migration_version: i64,
) -> AppResult<BackupManifest> {
    let mut entries = Vec::new();
    entries.push(read_entry(BACKUP_DB_PATH, db_path)?);

    if include_files && files_dir.exists() {
        for entry in WalkDir::new(files_dir).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                let size = entry.metadata().map(|metadata| metadata.len()).unwrap_or(0);
                if size as usize > MAX_CLINICAL_FILE_BYTES {
                    continue;
                }
                let relative = entry
                    .path()
                    .strip_prefix(files_dir)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .replace('\\', "/");
                entries.push(read_entry(&format!("files/{relative}"), entry.path())?);
            }
        }
    }

    let system_info = json!({
        "product": "Dentista v1 Professional",
        "appVersion": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "createdAt": now_utc()
    });
    let system_info_bytes = serde_json::to_vec_pretty(&system_info)?;
    entries.push(BackupEntry {
        zip_path: SYSTEM_INFO_PATH.to_string(),
        checksum_sha256: sha256_bytes(&system_info_bytes),
        bytes: system_info_bytes,
    });

    let checksums_without_manifest = checksums_text(&entries);
    let content_checksum = sha256_bytes(checksums_without_manifest.as_bytes());
    let file_count = entries
        .iter()
        .filter(|entry| entry.zip_path.starts_with("files/"))
        .count() as i64;
    let manifest = BackupManifest {
        backup_id: backup_id.to_string(),
        clinic_id: Some(clinic_id.to_string()),
        created_at: now_utc(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        database_version: "sqlite".to_string(),
        migration_version,
        includes: if include_files {
            vec!["database".to_string(), "files".to_string()]
        } else {
            vec!["database".to_string()]
        },
        table_counts,
        file_count,
        checksum: json!({
            "algorithm": "SHA-256",
            "contentSha256": content_checksum
        }),
        compression: "deflate".to_string(),
        encrypted: false,
        created_by_user_id: Some(created_by_user_id.to_string()),
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    let manifest_entry = BackupEntry {
        zip_path: MANIFEST_PATH.to_string(),
        checksum_sha256: sha256_bytes(&manifest_bytes),
        bytes: manifest_bytes,
    };
    let mut all_entries = vec![manifest_entry];
    all_entries.extend(entries);
    let checksums = checksums_text(&all_entries);

    let file = File::create(backup_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for entry in &all_entries {
        zip.start_file(&entry.zip_path, options)?;
        zip.write_all(&entry.bytes)?;
    }
    zip.start_file(CHECKSUMS_PATH, options)?;
    zip.write_all(checksums.as_bytes())?;
    zip.finish()?;
    Ok(manifest)
}

fn verify_backup_path(path: &Path) -> AppResult<BackupVerificationResult> {
    let path_text = path.to_string_lossy().to_string();
    let mut errors = Vec::new();
    if !path.exists() {
        errors.push("El archivo de respaldo no existe".to_string());
        return Ok(BackupVerificationResult {
            valid: false,
            path: path_text,
            backup_id: None,
            archive_checksum_sha256: None,
            manifest: None,
            checked_files: 0,
            errors,
        });
    }

    let archive_checksum = sha256_file(path).ok();
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let manifest = read_zip_json::<BackupManifest>(&mut archive, MANIFEST_PATH);
    let manifest = match manifest {
        Ok(value) => Some(value),
        Err(error) => {
            errors.push(format!("No se pudo leer manifest.json: {error}"));
            None
        }
    };
    if archive.by_name(BACKUP_DB_PATH).is_err() {
        errors.push("El respaldo no contiene data/dentalcare.sqlite".to_string());
    }
    let checksums = match read_zip_string(&mut archive, CHECKSUMS_PATH) {
        Ok(value) => value,
        Err(error) => {
            errors.push(format!("No se pudo leer checksums.sha256: {error}"));
            String::new()
        }
    };

    let mut checked_files = 0;
    for line in checksums.lines().filter(|line| !line.trim().is_empty()) {
        let Some((expected, name)) = line.split_once("  ") else {
            errors.push(format!("Línea de checksum inválida: {line}"));
            continue;
        };
        if !safe_zip_name(name) {
            errors.push(format!("Ruta insegura dentro del ZIP: {name}"));
            continue;
        }
        match archive.by_name(name) {
            Ok(mut file) => {
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;
                let actual = sha256_bytes(&bytes);
                if actual != expected {
                    errors.push(format!("Checksum inválido para {name}"));
                }
                checked_files += 1;
            }
            Err(_) => errors.push(format!("Archivo listado en checksum no existe: {name}")),
        }
    }

    let backup_id = manifest.as_ref().map(|manifest| manifest.backup_id.clone());
    Ok(BackupVerificationResult {
        valid: errors.is_empty(),
        path: path_text,
        backup_id,
        archive_checksum_sha256: archive_checksum,
        manifest,
        checked_files,
        errors,
    })
}

async fn backup_settings_for_clinic(
    state: &AppState,
    clinic_id: &str,
) -> AppResult<BackupSettings> {
    let row: Option<(i64, String, i64, i64, i64, String)> = sqlx::query_as(
        r#"
        SELECT automatic_enabled, frequency, include_files, encrypt_backups, retention_limit, updated_at
        FROM backup_settings
        WHERE clinic_id = ?
        "#,
    )
    .bind(clinic_id)
    .fetch_optional(&state.db)
    .await?;
    Ok(row
        .map(
            |(
                automatic_enabled,
                frequency,
                include_files,
                encrypt_backups,
                retention_limit,
                updated_at,
            )| BackupSettings {
                automatic_enabled: automatic_enabled == 1,
                frequency,
                include_files: include_files == 1,
                encrypt_backups: encrypt_backups == 1,
                retention_limit,
                updated_at: Some(updated_at),
            },
        )
        .unwrap_or(BackupSettings {
            automatic_enabled: true,
            frequency: "daily".to_string(),
            include_files: true,
            encrypt_backups: false,
            retention_limit: 30,
            updated_at: None,
        }))
}

async fn collect_table_counts(db: &sqlx::SqlitePool, clinic_id: &str) -> AppResult<Value> {
    let tables = [
        "patients",
        "appointments",
        "clinical_records",
        "clinical_evolutions",
        "odontogram_records",
        "treatment_plans",
        "estimates",
        "payments",
        "inventory_items",
        "files",
        "alerts",
        "users",
        "backups",
    ];
    let mut counts = serde_json::Map::new();
    for table in tables {
        let sql = format!("SELECT COUNT(*) FROM {table} WHERE clinic_id = ?");
        let count: i64 = sqlx::query_scalar(&sql)
            .bind(clinic_id)
            .fetch_one(db)
            .await
            .unwrap_or(0);
        counts.insert(table.to_string(), json!(count));
    }
    Ok(Value::Object(counts))
}

async fn migration_version(db: &sqlx::SqlitePool) -> AppResult<i64> {
    let version: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations")
        .fetch_one(db)
        .await
        .unwrap_or(0);
    Ok(version)
}

fn read_entry(zip_path: &str, path: &Path) -> AppResult<BackupEntry> {
    let bytes = fs::read(path)?;
    Ok(BackupEntry {
        zip_path: zip_path.to_string(),
        checksum_sha256: sha256_bytes(&bytes),
        bytes,
    })
}

fn checksums_text(entries: &[BackupEntry]) -> String {
    let mut lines = entries
        .iter()
        .map(|entry| format!("{}  {}", entry.checksum_sha256, entry.zip_path))
        .collect::<Vec<_>>();
    lines.sort();
    format!("{}\n", lines.join("\n"))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn sha256_file(path: &Path) -> AppResult<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn read_zip_string(archive: &mut ZipArchive<File>, name: &str) -> AppResult<String> {
    let mut file = archive.by_name(name)?;
    let mut value = String::new();
    file.read_to_string(&mut value)?;
    Ok(value)
}

fn read_zip_json<T: serde::de::DeserializeOwned>(
    archive: &mut ZipArchive<File>,
    name: &str,
) -> AppResult<T> {
    let text = read_zip_string(archive, name)?;
    Ok(serde_json::from_str(&text)?)
}

fn extract_zip_file(zip_path: &Path, entry_name: &str, target: &Path) -> AppResult<()> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut entry = archive.by_name(entry_name)?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut output = File::create(target)?;
    std::io::copy(&mut entry, &mut output)?;
    Ok(())
}

fn extract_zip_prefix(zip_path: &Path, prefix: &str, target_dir: &Path) -> AppResult<()> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_string();
        if !name.starts_with(prefix) || !safe_zip_name(&name) || entry.is_dir() {
            continue;
        }
        let relative = name.trim_start_matches(prefix);
        if relative.is_empty() || !safe_zip_name(relative) {
            continue;
        }
        let target = target_dir.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output = File::create(target)?;
        std::io::copy(&mut entry, &mut output)?;
    }
    Ok(())
}

fn zip_contains_prefix(zip_path: &Path, prefix: &str) -> AppResult<bool> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    for index in 0..archive.len() {
        if archive.by_index(index)?.name().starts_with(prefix) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn safe_zip_name(name: &str) -> bool {
    let path = Path::new(name);
    !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

fn preserve_existing_database(data_dir: &Path, safety_dir: &Path) -> AppResult<()> {
    let db_path = data_dir.join("dentalcare.sqlite");
    if !db_path.exists() {
        return Ok(());
    }
    let db_safety = safety_dir.join("data");
    fs::create_dir_all(&db_safety)?;
    for suffix in ["", "-wal", "-shm"] {
        let source = PathBuf::from(format!("{}{}", db_path.to_string_lossy(), suffix));
        if source.exists() {
            let file_name = source
                .file_name()
                .ok_or_else(|| AppError::Validation("Ruta de base inválida".to_string()))?;
            fs::copy(&source, db_safety.join(file_name))?;
        }
    }
    Ok(())
}

fn preserve_existing_files(files_dir: &Path, safety_dir: &Path) -> AppResult<()> {
    if !files_dir.exists() {
        return Ok(());
    }
    let target = safety_dir.join("files");
    fs::create_dir_all(&target)?;
    copy_dir(files_dir, &target)?;
    Ok(())
}

fn copy_dir(from: &Path, to: &Path) -> AppResult<()> {
    for entry in WalkDir::new(from).into_iter().filter_map(Result::ok) {
        let relative = entry.path().strip_prefix(from).unwrap_or(entry.path());
        let target = to.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn remove_sqlite_files(db_path: &Path) -> AppResult<()> {
    for suffix in ["", "-wal", "-shm"] {
        let path = PathBuf::from(format!("{}{}", db_path.to_string_lossy(), suffix));
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn parse_datetime(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|date| Utc.from_utc_datetime(&date))
        })
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S")
                .ok()
                .map(|date| Utc.from_utc_datetime(&date))
        })
}
