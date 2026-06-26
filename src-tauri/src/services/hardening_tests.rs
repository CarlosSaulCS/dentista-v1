use std::fs;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::database::{AppState, MIGRATOR};
use crate::models::{
    CreateAppointmentInput, CreateInventoryItemInput, CreateInventoryMovementInput,
    CreatePatientInput, ListAppointmentsInput, ListPatientsInput, LoginInput, SetupInput,
    UpdateAppointmentInput, UpdateInventoryItemInput, UpdatePatientInput,
};
use crate::services::{
    appointment_service, auth_service, backup_service, license_service, office_service,
    patient_service,
};
use crate::utils::new_id;

async fn test_pool() -> SqlitePool {
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
    pool
}

async fn test_session(pool: &SqlitePool) -> String {
    let auth = auth_service::setup_clinic_and_admin(
        pool,
        SetupInput {
            clinic_name: "Clinica Test".to_string(),
            clinic_phone: None,
            clinic_whatsapp: None,
            admin_full_name: "Admin Test".to_string(),
            admin_username: "admin".to_string(),
            admin_email: None,
            admin_password: "admin12345".to_string(),
        },
    )
    .await
    .expect("bootstrap admin session");

    auth.session_token
}

async fn create_patient(pool: &SqlitePool, session_token: &str) -> String {
    let patient = patient_service::create_patient(
        pool,
        session_token,
        CreatePatientInput {
            full_name: "Paciente Prueba".to_string(),
            birth_date: None,
            sex: None,
            phone: Some("5551112222".to_string()),
            whatsapp: None,
            email: Some("paciente@example.com".to_string()),
            address: None,
            emergency_contact_name: None,
            emergency_contact_phone: None,
            occupation: None,
            allergies: Some("Ninguna".to_string()),
            systemic_diseases: None,
            current_medications: None,
            relevant_history: None,
            habits: None,
            general_notes: None,
        },
    )
    .await
    .expect("create patient");

    patient.id
}

#[tokio::test]
async fn expired_trial_allows_read_only_and_dev_activation_unlocks_writes() {
    let root = std::env::temp_dir().join(format!("dv1-readonly-license-{}", new_id()));
    fs::create_dir_all(&root).expect("create readonly test root");
    let db_path = root.join("readonly.sqlite");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&db_path)
                .create_if_missing(true)
                .foreign_keys(true),
        )
        .await
        .expect("connect readonly file sqlite");
    MIGRATOR.run(&pool).await.expect("run migrations");
    let session_token = test_session(&pool).await;

    sqlx::query(
        "UPDATE app_license SET trial_ends_at = '2000-01-01T00:00:00+00:00', activated_at = NULL, activated_by_user_id = NULL",
    )
    .execute(&pool)
    .await
    .expect("expire trial");

    let read_only_login = auth_service::login(
        &pool,
        LoginInput {
            username: "admin".to_string(),
            password: "admin12345".to_string(),
        },
    )
    .await
    .expect("expired trial allows normal login in read-only mode");
    assert!(!read_only_login.license.can_write);
    assert_eq!(read_only_login.license.access_mode, "read_only");

    patient_service::list_patients(
        &pool,
        &session_token,
        ListPatientsInput {
            search: None,
            limit: Some(10),
        },
    )
    .await
    .expect("expired trial allows reads");

    let blocked_write = patient_service::create_patient(
        &pool,
        &session_token,
        CreatePatientInput {
            full_name: "Paciente Bloqueado".to_string(),
            birth_date: None,
            sex: None,
            phone: Some("5553334444".to_string()),
            whatsapp: None,
            email: None,
            address: None,
            emergency_contact_name: None,
            emergency_contact_phone: None,
            occupation: None,
            allergies: None,
            systemic_diseases: None,
            current_medications: None,
            relevant_history: None,
            habits: None,
            general_notes: None,
        },
    )
    .await
    .expect_err("expired trial blocks writes");
    assert!(blocked_write.to_string().contains("lectura"));

    let state = AppState {
        db: pool.clone(),
        app_data_dir: root.join("app"),
        files_dir: root.join("app").join("files"),
        backups_dir: root.join("backups"),
        reports_dir: root.join("reports"),
    };
    let backup = backup_service::create_backup(&state, &session_token)
        .await
        .expect("read-only license still allows manual backup");
    let verification = backup_service::verify_backup(&state, &session_token, &backup.path)
        .await
        .expect("read-only license still allows backup verification");
    assert!(verification.valid, "{:?}", verification.errors);

    license_service::dev_activate_legacy_local_license(&pool, None)
        .await
        .expect("dev activation unlocks local writes");

    let activated = auth_service::login(
        &pool,
        LoginInput {
            username: "admin".to_string(),
            password: "admin12345".to_string(),
        },
    )
    .await
    .expect("normal user password works after activation");
    assert!(activated.license.can_write);

    drop(pool);
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn patients_can_be_created_updated_listed_and_soft_deleted() {
    let pool = test_pool().await;
    let session_token = test_session(&pool).await;
    let patient_id = create_patient(&pool, &session_token).await;

    let updated = patient_service::update_patient(
        &pool,
        &session_token,
        UpdatePatientInput {
            id: patient_id.clone(),
            full_name: "Paciente Actualizado".to_string(),
            birth_date: Some("1990-01-01".to_string()),
            sex: Some("otro".to_string()),
            phone: Some("5559990000".to_string()),
            whatsapp: Some("5559990000".to_string()),
            email: Some("actualizado@example.com".to_string()),
            address: Some("Calle 1".to_string()),
            emergency_contact_name: Some("Contacto".to_string()),
            emergency_contact_phone: Some("5550001111".to_string()),
            occupation: Some("Docente".to_string()),
            allergies: Some("Penicilina".to_string()),
            systemic_diseases: Some("Ninguna".to_string()),
            current_medications: Some("Ninguno".to_string()),
            relevant_history: Some("Sin antecedentes".to_string()),
            habits: Some("Cepillado diario".to_string()),
            general_notes: Some("Paciente puntual".to_string()),
            status: "active".to_string(),
        },
    )
    .await
    .expect("update patient");

    assert_eq!(updated.full_name, "Paciente Actualizado");
    assert_eq!(updated.address.as_deref(), Some("Calle 1"));

    let listed = patient_service::list_patients(
        &pool,
        &session_token,
        ListPatientsInput {
            search: Some("Actualizado".to_string()),
            limit: Some(10),
        },
    )
    .await
    .expect("list patients");
    assert_eq!(listed.len(), 1);

    patient_service::soft_delete_patient(&pool, &session_token, &patient_id)
        .await
        .expect("soft delete patient");
    let listed_after_delete = patient_service::list_patients(
        &pool,
        &session_token,
        ListPatientsInput {
            search: Some("Actualizado".to_string()),
            limit: Some(10),
        },
    )
    .await
    .expect("list patients after delete");
    assert!(listed_after_delete.is_empty());
}

#[tokio::test]
async fn appointments_can_be_created_updated_statused_and_soft_deleted() {
    let pool = test_pool().await;
    let session_token = test_session(&pool).await;
    let patient_id = create_patient(&pool, &session_token).await;

    let appointment = appointment_service::create_appointment(
        &pool,
        &session_token,
        CreateAppointmentInput {
            patient_id: patient_id.clone(),
            dentist_user_id: None,
            starts_at: "2030-01-10T10:00".to_string(),
            duration_minutes: 45,
            reason: "Revision".to_string(),
            appointment_type: "revision".to_string(),
            notes: None,
        },
    )
    .await
    .expect("create appointment");

    let listed = appointment_service::list_appointments(
        &pool,
        &session_token,
        ListAppointmentsInput {
            date: Some("2030-01-10".to_string()),
            status: None,
        },
    )
    .await
    .expect("list appointments");
    assert_eq!(listed.len(), 1);

    let updated = appointment_service::update_appointment(
        &pool,
        &session_token,
        UpdateAppointmentInput {
            id: appointment.id.clone(),
            patient_id,
            dentist_user_id: None,
            starts_at: "2030-01-10T11:00".to_string(),
            duration_minutes: 30,
            reason: "Limpieza".to_string(),
            appointment_type: "limpieza".to_string(),
            status: "confirmada".to_string(),
            notes: Some("Confirmada por telefono".to_string()),
        },
    )
    .await
    .expect("update appointment");
    assert_eq!(updated.status, "confirmada");
    assert_eq!(updated.starts_at, "2030-01-10T11:00:00");

    appointment_service::soft_delete_appointment(&pool, &session_token, &appointment.id)
        .await
        .expect("soft delete appointment");
    let listed_after_delete = appointment_service::list_appointments(
        &pool,
        &session_token,
        ListAppointmentsInput {
            date: Some("2030-01-10".to_string()),
            status: None,
        },
    )
    .await
    .expect("list appointments after delete");
    assert_eq!(listed_after_delete.len(), 0);

    let deleted_status: (String, Option<String>) =
        sqlx::query_as("SELECT status, deleted_at FROM appointments WHERE id = ?")
            .bind(&appointment.id)
            .fetch_one(&pool)
            .await
            .expect("fetch soft deleted appointment");
    assert_eq!(deleted_status.0, "cancelada");
    assert!(deleted_status.1.is_some());
}

#[tokio::test]
async fn inventory_items_can_be_created_updated_moved_and_soft_deleted() {
    let pool = test_pool().await;
    let session_token = test_session(&pool).await;

    let item = office_service::create_inventory_item(
        &pool,
        &session_token,
        CreateInventoryItemInput {
            supplier_id: None,
            name: "Guantes nitrilo".to_string(),
            category: "Consumibles".to_string(),
            unit: "caja".to_string(),
            current_quantity: 3.0,
            minimum_stock: 2.0,
            cost_cents: 12000,
            purchase_date: None,
            expiration_date: None,
            location: Some("Almacen".to_string()),
        },
    )
    .await
    .expect("create inventory item");

    let updated = office_service::update_inventory_item(
        &pool,
        &session_token,
        UpdateInventoryItemInput {
            id: item.id.clone(),
            supplier_id: None,
            name: "Guantes nitrilo azul".to_string(),
            category: "Consumibles".to_string(),
            unit: "caja".to_string(),
            current_quantity: 8.0,
            minimum_stock: 3.0,
            cost_cents: 12500,
            purchase_date: None,
            expiration_date: None,
            location: Some("Gabinete A".to_string()),
            active: true,
        },
    )
    .await
    .expect("update inventory item");
    assert_eq!(updated.current_quantity, 8.0);
    assert_eq!(updated.location.as_deref(), Some("Gabinete A"));

    let moved = office_service::create_inventory_movement(
        &pool,
        &session_token,
        CreateInventoryMovementInput {
            inventory_item_id: item.id.clone(),
            movement_type: "salida".to_string(),
            quantity: 2.0,
            reason: Some("Uso en consulta".to_string()),
        },
    )
    .await
    .expect("create inventory movement");
    assert_eq!(moved.current_quantity, 6.0);

    office_service::soft_delete_inventory_item(&pool, &session_token, &item.id)
        .await
        .expect("soft delete inventory item");
    let active_items = office_service::list_inventory_items(&pool, &session_token)
        .await
        .expect("list inventory after delete");
    assert!(active_items.is_empty());
}

#[tokio::test]
async fn backup_creates_verifiable_zip_with_manifest_and_checksums() {
    let root = std::env::temp_dir().join(format!("dentalcare-backup-test-{}", new_id()));
    fs::create_dir_all(&root).expect("create test root");
    let db_path = root.join("test.sqlite");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&db_path)
                .create_if_missing(true)
                .foreign_keys(true),
        )
        .await
        .expect("connect file sqlite");
    MIGRATOR.run(&pool).await.expect("run migrations");
    let session_token = test_session(&pool).await;
    let patient_id = create_patient(&pool, &session_token).await;
    let app_data_dir = root.join("app");
    let files_dir = app_data_dir.join("files");
    let backups_dir = root.join("backups");
    let reports_dir = root.join("reports");
    let patient_dir = files_dir.join(&patient_id).join("Radiografias");
    fs::create_dir_all(&patient_dir).expect("create patient file dir");
    fs::write(patient_dir.join("rx.pdf"), b"PDF test").expect("write patient file");

    let state = AppState {
        db: pool,
        app_data_dir,
        files_dir,
        backups_dir,
        reports_dir,
    };

    let backup = backup_service::create_backup(&state, &session_token)
        .await
        .expect("create backup");
    assert!(backup.path.ends_with(".zip"));
    assert!(backup.checksum_sha256.is_some());

    let verification = backup_service::verify_backup(&state, &session_token, &backup.path)
        .await
        .expect("verify backup");
    assert!(verification.valid, "{:?}", verification.errors);
    assert_eq!(
        verification
            .manifest
            .as_ref()
            .map(|manifest| manifest.backup_id.as_str()),
        Some(backup.id.as_str())
    );
    assert!(verification.checked_files >= 3);

    let _ = fs::remove_dir_all(root);
}
