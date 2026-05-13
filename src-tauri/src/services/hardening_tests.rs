use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

use crate::database::MIGRATOR;
use crate::models::{
    CreateAppointmentInput, CreateInventoryItemInput, CreateInventoryMovementInput,
    CreatePatientInput, ListAppointmentsInput, ListPatientsInput, LoginInput, SetupInput,
    UpdateAppointmentInput, UpdateInventoryItemInput, UpdatePatientInput,
};
use crate::services::{appointment_service, auth_service, office_service, patient_service};

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
async fn expired_trial_blocks_normal_login_and_activation_unlocks_access() {
    let pool = test_pool().await;
    let session_token = test_session(&pool).await;

    sqlx::query(
        "UPDATE app_license SET trial_ends_at = '2000-01-01T00:00:00+00:00', activated_at = NULL, activated_by_user_id = NULL",
    )
    .execute(&pool)
    .await
    .expect("expire trial");

    let blocked_login = auth_service::login(
        &pool,
        LoginInput {
            username: "admin".to_string(),
            password: "admin12345".to_string(),
        },
    )
    .await
    .expect_err("expired trial blocks normal password");
    assert!(blocked_login.to_string().contains("prueba de 30 días"));

    let blocked_session = patient_service::list_patients(
        &pool,
        &session_token,
        ListPatientsInput {
            search: None,
            limit: Some(10),
        },
    )
    .await
    .expect_err("expired trial blocks existing sessions");
    assert!(blocked_session.to_string().contains("prueba de 30 días"));

    let activated = auth_service::login(
        &pool,
        LoginInput {
            username: "admin".to_string(),
            password: "Casc+10098".to_string(),
        },
    )
    .await
    .expect("activation key unlocks system");
    assert!(activated.license.is_licensed);

    auth_service::login(
        &pool,
        LoginInput {
            username: "admin".to_string(),
            password: "admin12345".to_string(),
        },
    )
    .await
    .expect("normal user password works after activation");
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
    assert!(listed_after_delete.is_empty());
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
