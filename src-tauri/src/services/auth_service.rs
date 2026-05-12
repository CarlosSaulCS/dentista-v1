use std::collections::HashMap;

use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::{FromRow, SqlitePool};

use crate::errors::{AppError, AppResult};
use crate::models::{
    AuthSession, BootstrapStatus, ClinicSummary, LoginInput, SetupInput, UserListItem, UserProfile,
};
use crate::security::{hash_password, hash_session_token, verify_password};
use crate::services::audit_service::log_action;
use crate::utils::{new_id, now_utc};

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub clinic_id: String,
    pub full_name: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, FromRow)]
struct UserAuthRow {
    id: String,
    clinic_id: String,
    full_name: String,
    username: String,
    email: Option<String>,
    password_hash: String,
    status: String,
    role_name: Option<String>,
    professional_license: Option<String>,
    specialty: Option<String>,
}

pub async fn get_bootstrap_status(db: &SqlitePool) -> AppResult<BootstrapStatus> {
    let users_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
            .fetch_one(db)
            .await?;

    let clinic = sqlx::query_as::<_, ClinicSummary>(
        r#"
        SELECT id, name, subtitle, phone, whatsapp, email, address
        FROM clinics
        ORDER BY created_at
        LIMIT 1
        "#,
    )
    .fetch_optional(db)
    .await?;

    Ok(BootstrapStatus {
        requires_setup: users_count == 0,
        clinic,
    })
}

pub async fn setup_clinic_and_admin(db: &SqlitePool, input: SetupInput) -> AppResult<AuthSession> {
    if input.clinic_name.trim().is_empty() {
        return Err(AppError::Validation(
            "El nombre del consultorio es obligatorio".to_string(),
        ));
    }
    if input.admin_full_name.trim().is_empty() || input.admin_username.trim().is_empty() {
        return Err(AppError::Validation(
            "El nombre y usuario del administrador son obligatorios".to_string(),
        ));
    }

    let users_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
            .fetch_one(db)
            .await?;
    if users_count > 0 {
        return Err(AppError::Conflict(
            "El sistema ya tiene un usuario administrador configurado".to_string(),
        ));
    }

    let now = now_utc();
    let clinic_id = new_id();
    let admin_role_id = new_id();
    let admin_user_id = new_id();
    let password_hash = hash_password(&input.admin_password)?;

    let mut tx = db.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO clinics (id, name, phone, whatsapp, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&clinic_id)
    .bind(input.clinic_name.trim())
    .bind(input.clinic_phone.as_deref().map(str::trim))
    .bind(input.clinic_whatsapp.as_deref().map(str::trim))
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    let role_specs = [
        (
            &admin_role_id,
            "Administrador",
            "administrator",
            "Acceso total al sistema",
        ),
        (
            &new_id(),
            "Odontólogo",
            "dentist",
            "Operación clínica y odontológica",
        ),
        (
            &new_id(),
            "Recepción",
            "reception",
            "Agenda, pacientes y atención inicial",
        ),
        (
            &new_id(),
            "Asistente",
            "assistant",
            "Apoyo clínico con permisos limitados",
        ),
        (&new_id(), "Caja", "cashier", "Pagos, recibos y caja"),
        (
            &new_id(),
            "Solo lectura",
            "readonly",
            "Consulta sin edición",
        ),
    ];

    for (role_id, name, system_key, description) in role_specs {
        sqlx::query(
            r#"
            INSERT INTO roles (id, clinic_id, name, system_key, description, is_system, created_at)
            VALUES (?, ?, ?, ?, ?, 1, ?)
            "#,
        )
        .bind(role_id)
        .bind(&clinic_id)
        .bind(name)
        .bind(system_key)
        .bind(description)
        .bind(&now)
        .execute(&mut *tx)
        .await?;
    }

    let permission_rows: Vec<(String, String)> = sqlx::query_as("SELECT id, code FROM permissions")
        .fetch_all(&mut *tx)
        .await?;
    let permission_map: HashMap<String, String> = permission_rows
        .into_iter()
        .map(|(id, code)| (code, id))
        .collect();

    for permission_id in permission_map.values() {
        sqlx::query("INSERT INTO role_permissions (role_id, permission_id) VALUES (?, ?)")
            .bind(&admin_role_id)
            .bind(permission_id)
            .execute(&mut *tx)
            .await?;
    }

    let role_permissions: [(&str, &[&str]); 5] = [
        (
            "dentist",
            &[
                "patients.view",
                "patients.create",
                "patients.edit",
                "clinical.view",
                "clinical.edit",
                "odontogram.view",
                "odontogram.edit",
                "appointments.create",
                "appointments.reschedule",
                "treatments.create",
            ],
        ),
        (
            "reception",
            &[
                "patients.view",
                "patients.create",
                "patients.edit",
                "appointments.create",
                "appointments.reschedule",
                "appointments.cancel",
            ],
        ),
        (
            "assistant",
            &[
                "patients.view",
                "clinical.view",
                "odontogram.view",
                "appointments.create",
            ],
        ),
        (
            "cashier",
            &[
                "patients.view",
                "payments.create",
                "payments.cancel",
                "reports.financial",
            ],
        ),
        (
            "readonly",
            &["patients.view", "clinical.view", "odontogram.view"],
        ),
    ];

    for (role_key, codes) in role_permissions {
        let role_id: String = sqlx::query_scalar(
            "SELECT id FROM roles WHERE clinic_id = ? AND system_key = ? LIMIT 1",
        )
        .bind(&clinic_id)
        .bind(role_key)
        .fetch_one(&mut *tx)
        .await?;

        for code in codes {
            if let Some(permission_id) = permission_map.get(*code) {
                sqlx::query(
                    "INSERT OR IGNORE INTO role_permissions (role_id, permission_id) VALUES (?, ?)",
                )
                .bind(&role_id)
                .bind(permission_id)
                .execute(&mut *tx)
                .await?;
            }
        }
    }

    sqlx::query(
        r#"
        INSERT INTO users
          (id, clinic_id, role_id, full_name, username, email, password_hash, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 'active', ?, ?)
        "#,
    )
    .bind(&admin_user_id)
    .bind(&clinic_id)
    .bind(&admin_role_id)
    .bind(input.admin_full_name.trim())
    .bind(input.admin_username.trim())
    .bind(input.admin_email.as_deref().map(str::trim))
    .bind(password_hash)
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    let folios = [("estimate", "COT"), ("payment", "PAG"), ("receipt", "REC")];
    for (entity, prefix) in folios {
        sqlx::query(
            "INSERT INTO folio_sequences (id, clinic_id, entity, prefix, next_number, updated_at) VALUES (?, ?, ?, ?, 1, ?)",
        )
        .bind(new_id())
        .bind(&clinic_id)
        .bind(entity)
        .bind(prefix)
        .bind(&now)
        .execute(&mut *tx)
        .await?;
    }

    let templates = [
        (
            "Recordatorio de cita",
            "Hola {{paciente}}, te recordamos tu cita en {{clinica}} el {{fecha}} a las {{hora}} con {{doctor}}.",
        ),
        (
            "Confirmación de cita",
            "Hola {{paciente}}, confirmamos tu cita para el {{fecha}} a las {{hora}}. Gracias por elegir {{clinica}}.",
        ),
        (
            "Aviso de saldo pendiente",
            "Hola {{paciente}}, tienes un saldo pendiente de {{saldo}} relacionado con {{tratamiento}}. Folio {{folio}}.",
        ),
    ];
    for (name, body) in templates {
        sqlx::query(
            "INSERT INTO message_templates (id, clinic_id, name, body, is_system, created_at, updated_at) VALUES (?, ?, ?, ?, 1, ?, ?)",
        )
        .bind(new_id())
        .bind(&clinic_id)
        .bind(name)
        .bind(body)
        .bind(&now)
        .bind(&now)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    log_action(
        db,
        Some(&clinic_id),
        Some(&admin_user_id),
        "bootstrap.create_admin",
        "users",
        Some(&admin_user_id),
        "security",
        Some(json!({ "username": input.admin_username })),
    )
    .await?;

    login(
        db,
        LoginInput {
            username: input.admin_username,
            password: input.admin_password,
        },
    )
    .await
}

pub async fn login(db: &SqlitePool, input: LoginInput) -> AppResult<AuthSession> {
    let username = input.username.trim();
    if username.is_empty() || input.password.is_empty() {
        return Err(AppError::Validation(
            "Usuario y contraseña son obligatorios".to_string(),
        ));
    }

    let user = sqlx::query_as::<_, UserAuthRow>(
        r#"
        SELECT u.id, u.clinic_id, u.full_name, u.username, u.email, u.password_hash, u.status,
               r.name AS role_name, u.professional_license, u.specialty
        FROM users u
        LEFT JOIN roles r ON r.id = u.role_id
        WHERE (u.username = ? OR u.email = ?) AND u.deleted_at IS NULL
        LIMIT 1
        "#,
    )
    .bind(username)
    .bind(username)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if user.status != "active" || !verify_password(&input.password, &user.password_hash) {
        return Err(AppError::Unauthorized);
    }

    let session_token = format!("{}.{}", new_id(), new_id());
    let token_hash = hash_session_token(&session_token);
    let expires_at = (Utc::now() + Duration::hours(12)).to_rfc3339();
    let now = now_utc();

    sqlx::query(
        "INSERT INTO user_sessions (id, user_id, token_hash, created_at, expires_at, last_seen_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(new_id())
    .bind(&user.id)
    .bind(token_hash)
    .bind(&now)
    .bind(&expires_at)
    .bind(&now)
    .execute(db)
    .await?;

    sqlx::query("UPDATE users SET last_login_at = ?, updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&now)
        .bind(&user.id)
        .execute(db)
        .await?;

    let permissions = load_user_permissions(db, &user.id).await?;

    log_action(
        db,
        Some(&user.clinic_id),
        Some(&user.id),
        "auth.login",
        "user_sessions",
        None,
        "info",
        Some(json!({ "username": user.username })),
    )
    .await?;

    Ok(AuthSession {
        session_token,
        expires_at,
        permissions,
        user: UserProfile {
            id: user.id,
            clinic_id: user.clinic_id,
            full_name: user.full_name,
            username: user.username,
            email: user.email,
            role_name: user.role_name,
            professional_license: user.professional_license,
            specialty: user.specialty,
        },
    })
}

pub async fn logout(db: &SqlitePool, session_token: &str) -> AppResult<()> {
    let token_hash = hash_session_token(session_token);
    let now = now_utc();
    sqlx::query(
        "UPDATE user_sessions SET revoked_at = ? WHERE token_hash = ? AND revoked_at IS NULL",
    )
    .bind(&now)
    .bind(token_hash)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn validate_session(
    db: &SqlitePool,
    session_token: &str,
    required_permission: Option<&str>,
) -> AppResult<AuthContext> {
    if session_token.trim().is_empty() {
        return Err(AppError::Unauthorized);
    }

    let token_hash = hash_session_token(session_token);
    let now = now_utc();
    let user = sqlx::query_as::<_, UserProfile>(
        r#"
        SELECT u.id, u.clinic_id, u.full_name, u.username, u.email, r.name AS role_name,
               u.professional_license, u.specialty
        FROM user_sessions s
        JOIN users u ON u.id = s.user_id
        LEFT JOIN roles r ON r.id = u.role_id
        WHERE s.token_hash = ?
          AND s.revoked_at IS NULL
          AND s.expires_at > ?
          AND u.status = 'active'
          AND u.deleted_at IS NULL
        LIMIT 1
        "#,
    )
    .bind(&token_hash)
    .bind(&now)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    sqlx::query("UPDATE user_sessions SET last_seen_at = ? WHERE token_hash = ?")
        .bind(&now)
        .bind(&token_hash)
        .execute(db)
        .await?;

    let permissions = load_user_permissions(db, &user.id).await?;
    if let Some(permission) = required_permission {
        if !permissions.iter().any(|item| item == permission) {
            return Err(AppError::PermissionDenied(permission.to_string()));
        }
    }

    Ok(AuthContext {
        user_id: user.id,
        clinic_id: user.clinic_id,
        full_name: user.full_name,
        permissions,
    })
}

pub async fn list_users(db: &SqlitePool, session_token: &str) -> AppResult<Vec<UserListItem>> {
    let ctx = validate_session(db, session_token, None).await?;
    let users = sqlx::query_as::<_, UserListItem>(
        r#"
        SELECT u.id, u.full_name, u.username, r.name AS role_name
        FROM users u
        LEFT JOIN roles r ON r.id = u.role_id
        WHERE u.clinic_id = ? AND u.status = 'active' AND u.deleted_at IS NULL
        ORDER BY u.full_name
        "#,
    )
    .bind(ctx.clinic_id)
    .fetch_all(db)
    .await?;

    Ok(users)
}

async fn load_user_permissions(db: &SqlitePool, user_id: &str) -> AppResult<Vec<String>> {
    let permissions: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT DISTINCT p.code
        FROM permissions p
        JOIN role_permissions rp ON rp.permission_id = p.id
        JOIN users u ON u.role_id = rp.role_id
        WHERE u.id = ?
        UNION
        SELECT DISTINCT p.code
        FROM permissions p
        JOIN user_permissions up ON up.permission_id = p.id
        WHERE up.user_id = ? AND up.allowed = 1
        ORDER BY code
        "#,
    )
    .bind(user_id)
    .bind(user_id)
    .fetch_all(db)
    .await?;

    Ok(permissions)
}
