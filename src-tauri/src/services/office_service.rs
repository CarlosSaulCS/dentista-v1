use std::fs;
use std::path::PathBuf;

use serde_json::json;
use sqlx::SqlitePool;

use crate::database::AppState;
use crate::errors::{AppError, AppResult};
use crate::models::*;
use crate::security::hash_password;
use crate::services::audit_service::log_action;
use crate::services::auth_service::{validate_session, validate_session_for_intent};
use crate::services::license_service::AccessIntent;
use crate::utils::{new_id, normalize_search, now_utc};

pub async fn list_treatments(
    db: &SqlitePool,
    session_token: &str,
) -> AppResult<Vec<TreatmentCatalogItem>> {
    let _ctx = validate_session(db, session_token, None).await?;
    let rows = sqlx::query_as::<_, TreatmentCatalogItem>(
        r#"
        SELECT id, name, category, description, base_price_cents, estimated_duration_minutes,
               requires_follow_up, active
        FROM treatment_catalog
        WHERE active = 1
        ORDER BY category, name
        "#,
    )
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn create_treatment(
    db: &SqlitePool,
    session_token: &str,
    input: CreateTreatmentInput,
) -> AppResult<TreatmentCatalogItem> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("treatments.create"),
        AccessIntent::DataWrite,
    )
    .await?;
    if input.name.trim().is_empty() || input.category.trim().is_empty() {
        return Err(AppError::Validation(
            "Nombre y categoría son obligatorios".to_string(),
        ));
    }
    if input.base_price_cents < 0 {
        return Err(AppError::Validation(
            "El precio no puede ser negativo".to_string(),
        ));
    }
    ensure_unique_treatment_name(
        db,
        &ctx.clinic_id,
        input.name.trim(),
        input.category.trim(),
        None,
    )
    .await?;

    let id = new_id();
    let now = now_utc();
    sqlx::query(
        r#"
        INSERT INTO treatment_catalog
          (id, clinic_id, name, category, description, base_price_cents, estimated_duration_minutes,
           requires_follow_up, active, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&ctx.clinic_id)
    .bind(input.name.trim())
    .bind(input.category.trim())
    .bind(input.description.as_deref().map(str::trim))
    .bind(input.base_price_cents)
    .bind(input.estimated_duration_minutes)
    .bind(if input.requires_follow_up { 1 } else { 0 })
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;

    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "treatments.create",
        "treatment_catalog",
        Some(&id),
        "info",
        Some(json!({ "name": input.name })),
    )
    .await?;

    get_treatment_by_id(db, &id).await
}

pub async fn update_treatment(
    db: &SqlitePool,
    session_token: &str,
    input: UpdateTreatmentInput,
) -> AppResult<TreatmentCatalogItem> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("treatments.create"),
        AccessIntent::DataWrite,
    )
    .await?;
    if input.id.trim().is_empty() {
        return Err(AppError::Validation(
            "Selecciona un tratamiento para editar".to_string(),
        ));
    }
    if input.name.trim().is_empty() || input.category.trim().is_empty() {
        return Err(AppError::Validation(
            "Nombre y categoría son obligatorios".to_string(),
        ));
    }
    if input.base_price_cents < 0 {
        return Err(AppError::Validation(
            "El precio no puede ser negativo".to_string(),
        ));
    }

    ensure_unique_treatment_name(
        db,
        &ctx.clinic_id,
        input.name.trim(),
        input.category.trim(),
        Some(input.id.trim()),
    )
    .await?;

    let now = now_utc();
    let result = sqlx::query(
        r#"
        UPDATE treatment_catalog
        SET name = ?, category = ?, description = ?, base_price_cents = ?,
            estimated_duration_minutes = ?, requires_follow_up = ?, active = ?, updated_at = ?
        WHERE id = ? AND (clinic_id = ? OR clinic_id IS NULL)
        "#,
    )
    .bind(input.name.trim())
    .bind(input.category.trim())
    .bind(input.description.as_deref().map(str::trim))
    .bind(input.base_price_cents)
    .bind(input.estimated_duration_minutes)
    .bind(if input.requires_follow_up { 1 } else { 0 })
    .bind(if input.active { 1 } else { 0 })
    .bind(&now)
    .bind(input.id.trim())
    .bind(&ctx.clinic_id)
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Validation(
            "Tratamiento no encontrado".to_string(),
        ));
    }

    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "treatments.update",
        "treatment_catalog",
        Some(input.id.trim()),
        "info",
        Some(json!({ "name": input.name, "active": input.active })),
    )
    .await?;

    get_treatment_by_id(db, input.id.trim()).await
}

pub async fn list_treatment_plans(
    db: &SqlitePool,
    session_token: &str,
) -> AppResult<Vec<TreatmentPlanSummary>> {
    let ctx = validate_session(db, session_token, None).await?;
    let rows = sqlx::query_as::<_, TreatmentPlanSummary>(
        r#"
        SELECT tp.id, tp.patient_id, p.full_name AS patient_name, tp.diagnosis, tp.subtotal_cents,
               tp.discount_cents, tp.total_cents,
               COALESCE((SELECT SUM(pa.amount_cents)
                         FROM payment_allocations pa
                         JOIN treatment_plan_items tpi ON tpi.id = pa.treatment_plan_item_id
                         WHERE tpi.treatment_plan_id = tp.id), 0) AS paid_cents,
               (tp.total_cents - COALESCE((SELECT SUM(pa.amount_cents)
                         FROM payment_allocations pa
                         JOIN treatment_plan_items tpi ON tpi.id = pa.treatment_plan_item_id
                         WHERE tpi.treatment_plan_id = tp.id), 0)) AS balance_cents,
               tp.status, tp.notes, tp.created_at
        FROM treatment_plans tp
        JOIN patients p ON p.id = tp.patient_id
        WHERE tp.clinic_id = ?
        ORDER BY tp.created_at DESC
        "#,
    )
    .bind(ctx.clinic_id)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn create_treatment_plan(
    db: &SqlitePool,
    session_token: &str,
    input: CreateTreatmentPlanInput,
) -> AppResult<TreatmentPlanSummary> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("treatments.create"),
        AccessIntent::DataWrite,
    )
    .await?;
    if input.patient_id.trim().is_empty() {
        return Err(AppError::Validation("Selecciona un paciente".to_string()));
    }
    if input.items.is_empty() {
        return Err(AppError::Validation(
            "Agrega al menos un tratamiento".to_string(),
        ));
    }

    let subtotal: i64 = input
        .items
        .iter()
        .map(|item| item.quantity.max(1) * item.unit_price_cents.max(0))
        .sum();
    let discount: i64 = input
        .items
        .iter()
        .map(|item| item.discount_cents.max(0))
        .sum();
    let total = (subtotal - discount).max(0);
    let plan_id = new_id();
    let now = now_utc();
    let mut tx = db.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO treatment_plans
          (id, clinic_id, patient_id, diagnosis, subtotal_cents, discount_cents, total_cents,
           status, notes, created_by_user_id, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 'draft', ?, ?, ?, ?)
        "#,
    )
    .bind(&plan_id)
    .bind(&ctx.clinic_id)
    .bind(&input.patient_id)
    .bind(input.diagnosis.as_deref().map(str::trim))
    .bind(subtotal)
    .bind(discount)
    .bind(total)
    .bind(input.notes.as_deref().map(str::trim))
    .bind(&ctx.user_id)
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    for item in &input.items {
        let item_total = (item.quantity.max(1) * item.unit_price_cents.max(0)
            - item.discount_cents.max(0))
        .max(0);
        sqlx::query(
            r#"
            INSERT INTO treatment_plan_items
              (id, clinic_id, treatment_plan_id, treatment_catalog_id, tooth_number, diagnosis,
               phase, priority, quantity, unit_price_cents, discount_cents, total_cents,
               status, notes, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'pending', ?, ?, ?)
            "#,
        )
        .bind(new_id())
        .bind(&ctx.clinic_id)
        .bind(&plan_id)
        .bind(
            item.treatment_catalog_id
                .as_deref()
                .filter(|value| !value.is_empty()),
        )
        .bind(item.tooth_number.as_deref().map(str::trim))
        .bind(item.diagnosis.as_deref().map(str::trim))
        .bind(item.phase.as_deref().map(str::trim))
        .bind(item.priority.as_deref().map(str::trim))
        .bind(item.quantity.max(1))
        .bind(item.unit_price_cents.max(0))
        .bind(item.discount_cents.max(0))
        .bind(item_total)
        .bind(item.notes.as_deref().map(str::trim))
        .bind(&now)
        .bind(&now)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "treatment_plans.create",
        "treatment_plans",
        Some(&plan_id),
        "info",
        Some(json!({ "patientId": input.patient_id, "totalCents": total })),
    )
    .await?;
    get_treatment_plan_by_id(db, &ctx.clinic_id, &plan_id).await
}

pub async fn list_treatment_plan_items(
    db: &SqlitePool,
    session_token: &str,
    treatment_plan_id: &str,
) -> AppResult<Vec<TreatmentPlanItemView>> {
    let ctx = validate_session(db, session_token, None).await?;
    let rows = sqlx::query_as::<_, TreatmentPlanItemView>(
        r#"
        SELECT tpi.id, tpi.treatment_plan_id, tc.name AS treatment_name, tpi.tooth_number,
               tpi.diagnosis, tpi.phase, tpi.priority, tpi.quantity, tpi.unit_price_cents,
               tpi.discount_cents, tpi.total_cents, tpi.status, tpi.notes
        FROM treatment_plan_items tpi
        LEFT JOIN treatment_catalog tc ON tc.id = tpi.treatment_catalog_id
        WHERE tpi.clinic_id = ? AND tpi.treatment_plan_id = ?
        ORDER BY tpi.created_at
        "#,
    )
    .bind(ctx.clinic_id)
    .bind(treatment_plan_id)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn list_estimates(
    db: &SqlitePool,
    session_token: &str,
) -> AppResult<Vec<EstimateSummary>> {
    let ctx = validate_session(db, session_token, None).await?;
    let rows = sqlx::query_as::<_, EstimateSummary>(
        r#"
        SELECT e.id, e.patient_id, p.full_name AS patient_name, e.treatment_plan_id, e.folio,
               e.status, e.valid_until, e.subtotal_cents, e.discount_cents, e.total_cents,
               e.observations, e.terms, e.created_at
        FROM estimates e
        JOIN patients p ON p.id = e.patient_id
        WHERE e.clinic_id = ?
        ORDER BY e.created_at DESC
        "#,
    )
    .bind(ctx.clinic_id)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn create_estimate(
    db: &SqlitePool,
    session_token: &str,
    input: CreateEstimateInput,
) -> AppResult<EstimateSummary> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("treatments.create"),
        AccessIntent::DataWrite,
    )
    .await?;
    if input.patient_id.trim().is_empty() || input.items.is_empty() {
        return Err(AppError::Validation(
            "Paciente y conceptos son obligatorios".to_string(),
        ));
    }
    let subtotal: i64 = input
        .items
        .iter()
        .map(|item| item.quantity.max(1) * item.unit_price_cents.max(0))
        .sum();
    let discount: i64 = input
        .items
        .iter()
        .map(|item| item.discount_cents.max(0))
        .sum();
    let total = (subtotal - discount).max(0);
    let estimate_id = new_id();
    let folio = next_folio(db, &ctx.clinic_id, "estimate").await?;
    let now = now_utc();
    let mut tx = db.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO estimates
          (id, clinic_id, patient_id, treatment_plan_id, folio, status, valid_until,
           subtotal_cents, discount_cents, total_cents, observations, terms,
           created_by_user_id, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, 'draft', ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&estimate_id)
    .bind(&ctx.clinic_id)
    .bind(&input.patient_id)
    .bind(
        input
            .treatment_plan_id
            .as_deref()
            .filter(|value| !value.is_empty()),
    )
    .bind(&folio)
    .bind(
        input
            .valid_until
            .as_deref()
            .filter(|value| !value.is_empty()),
    )
    .bind(subtotal)
    .bind(discount)
    .bind(total)
    .bind(input.observations.as_deref().map(str::trim))
    .bind(input.terms.as_deref().map(str::trim))
    .bind(&ctx.user_id)
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    for item in &input.items {
        let total = (item.quantity.max(1) * item.unit_price_cents.max(0)
            - item.discount_cents.max(0))
        .max(0);
        sqlx::query(
            r#"
            INSERT INTO estimate_items
              (id, clinic_id, estimate_id, treatment_catalog_id, description, quantity,
               unit_price_cents, discount_cents, total_cents)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(new_id())
        .bind(&ctx.clinic_id)
        .bind(&estimate_id)
        .bind(
            item.treatment_catalog_id
                .as_deref()
                .filter(|value| !value.is_empty()),
        )
        .bind(item.description.trim())
        .bind(item.quantity.max(1))
        .bind(item.unit_price_cents.max(0))
        .bind(item.discount_cents.max(0))
        .bind(total)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "estimates.create",
        "estimates",
        Some(&estimate_id),
        "info",
        Some(json!({ "folio": folio, "totalCents": total })),
    )
    .await?;
    get_estimate_by_id(db, &ctx.clinic_id, &estimate_id).await
}

pub async fn update_estimate_status(
    db: &SqlitePool,
    session_token: &str,
    input: UpdateStatusInput,
) -> AppResult<EstimateSummary> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("treatments.create"),
        AccessIntent::DataWrite,
    )
    .await?;
    let now = now_utc();
    sqlx::query("UPDATE estimates SET status = ?, updated_at = ? WHERE clinic_id = ? AND id = ?")
        .bind(&input.status)
        .bind(&now)
        .bind(&ctx.clinic_id)
        .bind(&input.id)
        .execute(db)
        .await?;
    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "estimates.status",
        "estimates",
        Some(&input.id),
        "info",
        Some(json!({ "status": input.status })),
    )
    .await?;
    get_estimate_by_id(db, &ctx.clinic_id, &input.id).await
}

pub async fn list_estimate_items(
    db: &SqlitePool,
    session_token: &str,
    estimate_id: &str,
) -> AppResult<Vec<EstimateItemView>> {
    let ctx = validate_session(db, session_token, None).await?;
    let rows = sqlx::query_as::<_, EstimateItemView>(
        r#"
        SELECT id, estimate_id, description, quantity, unit_price_cents, discount_cents, total_cents
        FROM estimate_items
        WHERE clinic_id = ? AND estimate_id = ?
        ORDER BY id
        "#,
    )
    .bind(ctx.clinic_id)
    .bind(estimate_id)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn list_payments(db: &SqlitePool, session_token: &str) -> AppResult<Vec<PaymentSummary>> {
    let ctx = validate_session(db, session_token, Some("payments.create")).await?;
    let rows = sqlx::query_as::<_, PaymentSummary>(
        r#"
        SELECT pmt.id, pmt.patient_id, p.full_name AS patient_name, pmt.folio, pmt.concept,
               pmt.amount_cents, pmt.method, pmt.status, pmt.paid_at,
               u.full_name AS received_by_name, pmt.notes,
               COALESCE((
                 SELECT COUNT(*)
                 FROM files f
                 WHERE f.clinic_id = pmt.clinic_id
                   AND f.related_entity_type = 'payments'
                   AND f.related_entity_id = pmt.id
                   AND f.deleted_at IS NULL
               ), 0) AS proof_files_count
        FROM payments pmt
        JOIN patients p ON p.id = pmt.patient_id
        JOIN users u ON u.id = pmt.received_by_user_id
        WHERE pmt.clinic_id = ?
        ORDER BY pmt.paid_at DESC
        "#,
    )
    .bind(ctx.clinic_id)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn register_payment(
    db: &SqlitePool,
    session_token: &str,
    input: RegisterPaymentInput,
) -> AppResult<PaymentSummary> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("payments.create"),
        AccessIntent::DataWrite,
    )
    .await?;
    if input.patient_id.trim().is_empty()
        || input.concept.trim().is_empty()
        || input.amount_cents <= 0
    {
        return Err(AppError::Validation(
            "Paciente, concepto y monto son obligatorios".to_string(),
        ));
    }

    let payment_id = new_id();
    let folio = next_folio(db, &ctx.clinic_id, "payment").await?;
    let now = now_utc();
    let mut tx = db.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO payments
          (id, clinic_id, patient_id, folio, concept, amount_cents, method, status, paid_at,
           received_by_user_id, notes, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 'active', ?, ?, ?, ?)
        "#,
    )
    .bind(&payment_id)
    .bind(&ctx.clinic_id)
    .bind(&input.patient_id)
    .bind(&folio)
    .bind(input.concept.trim())
    .bind(input.amount_cents)
    .bind(input.method.trim())
    .bind(&now)
    .bind(&ctx.user_id)
    .bind(input.notes.as_deref().map(str::trim))
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    if input.estimate_id.is_some() || input.treatment_plan_item_id.is_some() {
        sqlx::query(
            r#"
            INSERT INTO payment_allocations
              (id, clinic_id, payment_id, treatment_plan_item_id, estimate_id, amount_cents, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(new_id())
        .bind(&ctx.clinic_id)
        .bind(&payment_id)
        .bind(input.treatment_plan_item_id.as_deref().filter(|value| !value.is_empty()))
        .bind(input.estimate_id.as_deref().filter(|value| !value.is_empty()))
        .bind(input.amount_cents)
        .bind(&now)
        .execute(&mut *tx)
        .await?;
    }

    if let Some(cash_register_id) = current_cash_register_id_tx(&mut tx, &ctx.clinic_id).await? {
        sqlx::query(
            r#"
            INSERT INTO cash_movements
              (id, clinic_id, cash_register_id, payment_id, movement_type, category, amount_cents,
               method, description, created_by_user_id, created_at)
            VALUES (?, ?, ?, ?, 'entrada', 'pago', ?, ?, ?, ?, ?)
            "#,
        )
        .bind(new_id())
        .bind(&ctx.clinic_id)
        .bind(cash_register_id)
        .bind(&payment_id)
        .bind(input.amount_cents)
        .bind(input.method.trim())
        .bind(input.concept.trim())
        .bind(&ctx.user_id)
        .bind(&now)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "payments.create",
        "payments",
        Some(&payment_id),
        "finance",
        Some(json!({ "folio": folio, "amountCents": input.amount_cents })),
    )
    .await?;
    get_payment_by_id(db, &ctx.clinic_id, &payment_id).await
}

pub async fn get_current_cash_register(
    db: &SqlitePool,
    session_token: &str,
) -> AppResult<Option<CashRegisterSummary>> {
    let ctx = validate_session(db, session_token, Some("payments.create")).await?;
    let id: Option<String> = sqlx::query_scalar(
        "SELECT id FROM cash_registers WHERE clinic_id = ? AND status = 'open' ORDER BY opened_at DESC LIMIT 1",
    )
    .bind(&ctx.clinic_id)
    .fetch_optional(db)
    .await?;
    match id {
        Some(id) => get_cash_register_by_id(db, &ctx.clinic_id, &id)
            .await
            .map(Some),
        None => Ok(None),
    }
}

pub async fn open_cash_register(
    db: &SqlitePool,
    session_token: &str,
    input: OpenCashRegisterInput,
) -> AppResult<CashRegisterSummary> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("payments.create"),
        AccessIntent::DataWrite,
    )
    .await?;
    if get_current_cash_register(db, session_token)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("Ya existe una caja abierta".to_string()));
    }
    let id = new_id();
    let now = now_utc();
    sqlx::query(
        "INSERT INTO cash_registers (id, clinic_id, opened_by_user_id, opened_at, opening_float_cents, status) VALUES (?, ?, ?, ?, ?, 'open')",
    )
    .bind(&id)
    .bind(&ctx.clinic_id)
    .bind(&ctx.user_id)
    .bind(&now)
    .bind(input.opening_float_cents.max(0))
    .execute(db)
    .await?;
    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "cash.open",
        "cash_registers",
        Some(&id),
        "finance",
        None,
    )
    .await?;
    get_cash_register_by_id(db, &ctx.clinic_id, &id).await
}

pub async fn close_cash_register(
    db: &SqlitePool,
    session_token: &str,
    input: CloseCashRegisterInput,
) -> AppResult<CashClosureResult> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("reports.financial"),
        AccessIntent::DataWrite,
    )
    .await?;
    let register = get_cash_register_by_id(db, &ctx.clinic_id, &input.cash_register_id).await?;
    let expected = register.opening_float_cents + register.total_cash_cents;
    let difference = input.counted_cash_cents - expected;
    let closure_id = new_id();
    let now = now_utc();
    sqlx::query(
        "INSERT INTO cash_closures (id, clinic_id, cash_register_id, expected_cash_cents, counted_cash_cents, difference_cents, report_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&closure_id)
    .bind(&ctx.clinic_id)
    .bind(&input.cash_register_id)
    .bind(expected)
    .bind(input.counted_cash_cents)
    .bind(difference)
    .bind(json!({ "register": register }).to_string())
    .bind(&now)
    .execute(db)
    .await?;
    sqlx::query("UPDATE cash_registers SET status = 'closed', closed_at = ?, closed_by_user_id = ? WHERE clinic_id = ? AND id = ?")
        .bind(&now)
        .bind(&ctx.user_id)
        .bind(&ctx.clinic_id)
        .bind(&input.cash_register_id)
        .execute(db)
        .await?;
    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "cash.close",
        "cash_registers",
        Some(&input.cash_register_id),
        "finance",
        Some(json!({ "differenceCents": difference })),
    )
    .await?;
    Ok(CashClosureResult {
        cash_register_id: input.cash_register_id,
        expected_cash_cents: expected,
        counted_cash_cents: input.counted_cash_cents,
        difference_cents: difference,
    })
}

pub async fn list_suppliers(
    db: &SqlitePool,
    session_token: &str,
) -> AppResult<Vec<SupplierSummary>> {
    let ctx = validate_session(db, session_token, None).await?;
    let rows = sqlx::query_as::<_, SupplierSummary>(
        "SELECT id, name, phone, email, notes, active FROM suppliers WHERE clinic_id = ? ORDER BY name",
    )
    .bind(ctx.clinic_id)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn create_supplier(
    db: &SqlitePool,
    session_token: &str,
    input: CreateSupplierInput,
) -> AppResult<SupplierSummary> {
    let ctx = validate_session_for_intent(db, session_token, None, AccessIntent::DataWrite).await?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation(
            "El proveedor requiere nombre".to_string(),
        ));
    }
    let id = new_id();
    let now = now_utc();
    sqlx::query("INSERT INTO suppliers (id, clinic_id, name, phone, email, notes, active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?)")
        .bind(&id)
        .bind(&ctx.clinic_id)
        .bind(input.name.trim())
        .bind(input.phone.as_deref().map(str::trim))
        .bind(input.email.as_deref().map(str::trim))
        .bind(input.notes.as_deref().map(str::trim))
        .bind(&now)
        .bind(&now)
        .execute(db)
        .await?;
    get_supplier_by_id(db, &ctx.clinic_id, &id).await
}

pub async fn list_inventory_items(
    db: &SqlitePool,
    session_token: &str,
) -> AppResult<Vec<InventoryItemSummary>> {
    let ctx = validate_session(db, session_token, None).await?;
    let rows = sqlx::query_as::<_, InventoryItemSummary>(
        r#"
        SELECT ii.id, ii.supplier_id, s.name AS supplier_name, ii.name, ii.category, ii.unit,
               ii.current_quantity, ii.minimum_stock, ii.cost_cents, ii.expiration_date,
               ii.location, ii.active
        FROM inventory_items ii
        LEFT JOIN suppliers s ON s.id = ii.supplier_id
        WHERE ii.clinic_id = ?
          AND ii.deleted_at IS NULL
          AND ii.active = 1
        ORDER BY ii.name
        "#,
    )
    .bind(ctx.clinic_id)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn create_inventory_item(
    db: &SqlitePool,
    session_token: &str,
    input: CreateInventoryItemInput,
) -> AppResult<InventoryItemSummary> {
    let ctx = validate_session_for_intent(db, session_token, None, AccessIntent::DataWrite).await?;
    if input.name.trim().is_empty()
        || input.category.trim().is_empty()
        || input.unit.trim().is_empty()
    {
        return Err(AppError::Validation(
            "Nombre, categoría y unidad son obligatorios".to_string(),
        ));
    }
    if input.current_quantity < 0.0 || input.minimum_stock < 0.0 || input.cost_cents < 0 {
        return Err(AppError::Validation(
            "Cantidad, stock mínimo y costo no pueden ser negativos".to_string(),
        ));
    }
    ensure_unique_inventory_item(
        db,
        &ctx.clinic_id,
        input.name.trim(),
        input.category.trim(),
        input.unit.trim(),
        None,
    )
    .await?;
    let id = new_id();
    let now = now_utc();
    sqlx::query(
        r#"
        INSERT INTO inventory_items
          (id, clinic_id, supplier_id, name, category, unit, current_quantity, minimum_stock,
           cost_cents, purchase_date, expiration_date, location, active, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&ctx.clinic_id)
    .bind(
        input
            .supplier_id
            .as_deref()
            .filter(|value| !value.is_empty()),
    )
    .bind(input.name.trim())
    .bind(input.category.trim())
    .bind(input.unit.trim())
    .bind(input.current_quantity)
    .bind(input.minimum_stock)
    .bind(input.cost_cents.max(0))
    .bind(
        input
            .purchase_date
            .as_deref()
            .filter(|value| !value.is_empty()),
    )
    .bind(
        input
            .expiration_date
            .as_deref()
            .filter(|value| !value.is_empty()),
    )
    .bind(input.location.as_deref().map(str::trim))
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;
    let item = get_inventory_item_by_id(db, &ctx.clinic_id, &id).await?;
    reconcile_system_alerts(db, &ctx.clinic_id).await?;
    Ok(item)
}

pub async fn update_inventory_item(
    db: &SqlitePool,
    session_token: &str,
    input: UpdateInventoryItemInput,
) -> AppResult<InventoryItemSummary> {
    let ctx = validate_session_for_intent(db, session_token, None, AccessIntent::DataWrite).await?;
    if input.id.trim().is_empty() {
        return Err(AppError::Validation(
            "Selecciona un insumo para editar".to_string(),
        ));
    }
    if input.name.trim().is_empty()
        || input.category.trim().is_empty()
        || input.unit.trim().is_empty()
    {
        return Err(AppError::Validation(
            "Nombre, categoría y unidad son obligatorios".to_string(),
        ));
    }
    if input.current_quantity < 0.0 || input.minimum_stock < 0.0 || input.cost_cents < 0 {
        return Err(AppError::Validation(
            "Cantidad, stock mínimo y costo no pueden ser negativos".to_string(),
        ));
    }
    ensure_unique_inventory_item(
        db,
        &ctx.clinic_id,
        input.name.trim(),
        input.category.trim(),
        input.unit.trim(),
        Some(input.id.trim()),
    )
    .await?;

    let now = now_utc();
    let result = sqlx::query(
        r#"
        UPDATE inventory_items
        SET supplier_id = ?, name = ?, category = ?, unit = ?, current_quantity = ?,
            minimum_stock = ?, cost_cents = ?, purchase_date = ?, expiration_date = ?,
            location = ?, active = ?, updated_at = ?
        WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL
        "#,
    )
    .bind(
        input
            .supplier_id
            .as_deref()
            .filter(|value| !value.is_empty()),
    )
    .bind(input.name.trim())
    .bind(input.category.trim())
    .bind(input.unit.trim())
    .bind(input.current_quantity)
    .bind(input.minimum_stock)
    .bind(input.cost_cents)
    .bind(
        input
            .purchase_date
            .as_deref()
            .filter(|value| !value.is_empty()),
    )
    .bind(
        input
            .expiration_date
            .as_deref()
            .filter(|value| !value.is_empty()),
    )
    .bind(input.location.as_deref().map(str::trim))
    .bind(if input.active { 1 } else { 0 })
    .bind(&now)
    .bind(&ctx.clinic_id)
    .bind(input.id.trim())
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Validation("Insumo no encontrado".to_string()));
    }

    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "inventory.update",
        "inventory_items",
        Some(input.id.trim()),
        "info",
        Some(json!({ "name": input.name, "quantity": input.current_quantity, "minimumStock": input.minimum_stock })),
    )
    .await?;
    reconcile_system_alerts(db, &ctx.clinic_id).await?;
    get_inventory_item_by_id(db, &ctx.clinic_id, input.id.trim()).await
}

pub async fn soft_delete_inventory_item(
    db: &SqlitePool,
    session_token: &str,
    inventory_item_id: &str,
) -> AppResult<InventoryItemSummary> {
    let ctx = validate_session_for_intent(db, session_token, None, AccessIntent::DataWrite).await?;
    if inventory_item_id.trim().is_empty() {
        return Err(AppError::Validation(
            "Selecciona un insumo para dar de baja".to_string(),
        ));
    }
    let item = get_inventory_item_by_id(db, &ctx.clinic_id, inventory_item_id).await?;
    let now = now_utc();
    sqlx::query(
        r#"
        UPDATE inventory_items
        SET active = 0, deleted_at = ?, deleted_by_user_id = ?, updated_at = ?
        WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL
        "#,
    )
    .bind(&now)
    .bind(&ctx.user_id)
    .bind(&now)
    .bind(&ctx.clinic_id)
    .bind(inventory_item_id.trim())
    .execute(db)
    .await?;

    log_action(
        db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "inventory.soft_delete",
        "inventory_items",
        Some(inventory_item_id.trim()),
        "warning",
        Some(json!({ "name": item.name })),
    )
    .await?;
    reconcile_system_alerts(db, &ctx.clinic_id).await?;
    Ok(item)
}

pub async fn create_inventory_movement(
    db: &SqlitePool,
    session_token: &str,
    input: CreateInventoryMovementInput,
) -> AppResult<InventoryItemSummary> {
    let ctx = validate_session_for_intent(db, session_token, None, AccessIntent::DataWrite).await?;
    if input.inventory_item_id.trim().is_empty() || input.quantity <= 0.0 {
        return Err(AppError::Validation(
            "Insumo y cantidad son obligatorios".to_string(),
        ));
    }
    let direction = match input.movement_type.as_str() {
        "salida" | "merma" | "consumo" => -1.0,
        _ => 1.0,
    };
    let current_quantity: f64 = sqlx::query_scalar(
        "SELECT current_quantity FROM inventory_items WHERE clinic_id = ? AND id = ? AND active = 1 AND deleted_at IS NULL",
    )
    .bind(&ctx.clinic_id)
    .bind(input.inventory_item_id.trim())
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::Validation("Insumo no encontrado".to_string()))?;
    if direction < 0.0 && current_quantity < input.quantity {
        return Err(AppError::Validation(
            "No hay stock suficiente para registrar la salida".to_string(),
        ));
    }
    let now = now_utc();
    sqlx::query(
        "INSERT INTO inventory_movements (id, clinic_id, inventory_item_id, movement_type, quantity, reason, created_by_user_id, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(new_id())
    .bind(&ctx.clinic_id)
    .bind(&input.inventory_item_id)
    .bind(input.movement_type.trim())
    .bind(input.quantity)
    .bind(input.reason.as_deref().map(str::trim))
    .bind(&ctx.user_id)
    .bind(&now)
    .execute(db)
    .await?;
    sqlx::query("UPDATE inventory_items SET current_quantity = current_quantity + ?, updated_at = ? WHERE clinic_id = ? AND id = ?")
        .bind(input.quantity * direction)
        .bind(&now)
        .bind(&ctx.clinic_id)
        .bind(&input.inventory_item_id)
        .execute(db)
        .await?;
    let item = get_inventory_item_by_id(db, &ctx.clinic_id, &input.inventory_item_id).await?;
    reconcile_system_alerts(db, &ctx.clinic_id).await?;
    Ok(item)
}

pub async fn list_alerts(db: &SqlitePool, session_token: &str) -> AppResult<Vec<AlertSummary>> {
    let ctx = validate_session(db, session_token, None).await?;
    reconcile_system_alerts(db, &ctx.clinic_id).await?;
    let rows = sqlx::query_as::<_, AlertSummary>(
        r#"
        SELECT a.id, p.full_name AS patient_name, a.alert_type, a.priority, a.title,
               a.message, a.due_at, a.status, a.created_at
        FROM alerts a
        LEFT JOIN patients p ON p.id = a.patient_id
        WHERE a.clinic_id = ?
        ORDER BY CASE a.priority WHEN 'critica' THEN 0 WHEN 'alta' THEN 1 WHEN 'media' THEN 2 ELSE 3 END, a.created_at DESC
        "#,
    )
    .bind(ctx.clinic_id)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn create_alert(
    db: &SqlitePool,
    session_token: &str,
    input: CreateAlertInput,
) -> AppResult<AlertSummary> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("alerts.manage"),
        AccessIntent::DataWrite,
    )
    .await?;
    if input.title.trim().is_empty() || input.message.trim().is_empty() {
        return Err(AppError::Validation(
            "Título y mensaje son obligatorios".to_string(),
        ));
    }
    let id = new_id();
    let now = now_utc();
    sqlx::query(
        "INSERT INTO alerts (id, clinic_id, patient_id, alert_type, priority, title, message, due_at, status, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'open', ?)",
    )
    .bind(&id)
    .bind(&ctx.clinic_id)
    .bind(input.patient_id.as_deref().filter(|value| !value.is_empty()))
    .bind(input.alert_type.trim())
    .bind(input.priority.trim())
    .bind(input.title.trim())
    .bind(input.message.trim())
    .bind(input.due_at.as_deref().filter(|value| !value.is_empty()))
    .bind(&now)
    .execute(db)
    .await?;
    get_alert_by_id(db, &ctx.clinic_id, &id).await
}

pub async fn resolve_alert(
    db: &SqlitePool,
    session_token: &str,
    id: &str,
) -> AppResult<AlertSummary> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("alerts.manage"),
        AccessIntent::DataWrite,
    )
    .await?;
    let now = now_utc();
    sqlx::query(
        "UPDATE alerts SET status = 'resolved', resolved_at = ? WHERE clinic_id = ? AND id = ?",
    )
    .bind(&now)
    .bind(&ctx.clinic_id)
    .bind(id)
    .execute(db)
    .await?;
    get_alert_by_id(db, &ctx.clinic_id, id).await
}

pub async fn save_patient_file(
    state: &AppState,
    session_token: &str,
    input: SavePatientFileInput,
) -> AppResult<PatientFileSummary> {
    let required_permission = if input.related_entity_type.as_deref() == Some("payments") {
        Some("payments.create")
    } else {
        Some("patients.edit")
    };
    let ctx = validate_session_for_intent(
        &state.db,
        session_token,
        required_permission,
        AccessIntent::DataWrite,
    )
    .await?;
    if input.patient_id.trim().is_empty()
        || input.original_name.trim().is_empty()
        || input.bytes.is_empty()
    {
        return Err(AppError::Validation(
            "Paciente y archivo son obligatorios".to_string(),
        ));
    }
    if input.bytes.len() > 100 * 1024 * 1024 {
        return Err(AppError::Validation(
            "El archivo excede el límite de 100 MB".to_string(),
        ));
    }
    if !is_allowed_clinical_file(input.mime_type.as_deref(), &input.original_name) {
        return Err(AppError::Validation(
            "Tipo de archivo clínico no permitido".to_string(),
        ));
    }
    let category_id = ensure_file_category(&state.db, &input.category_name).await?;
    let file_id = new_id();
    let safe_name = sanitize_filename::sanitize(&input.original_name);
    let stored_name = format!("{}-{}", file_id, safe_name);
    let category_dir = sanitize_filename::sanitize(&input.category_name);
    let patient_dir = state.files_dir.join(&input.patient_id).join(&category_dir);
    fs::create_dir_all(&patient_dir)?;
    let absolute_path = patient_dir.join(&stored_name);
    fs::write(&absolute_path, &input.bytes)?;
    let relative_path = PathBuf::from(&input.patient_id)
        .join(&category_dir)
        .join(&stored_name)
        .to_string_lossy()
        .replace('\\', "/");
    let now = now_utc();
    let file_type = input
        .mime_type
        .as_deref()
        .and_then(|mime| mime.split('/').next_back())
        .unwrap_or("document")
        .to_string();

    sqlx::query(
        r#"
        INSERT INTO files
          (id, clinic_id, patient_id, category_id, file_type, original_name, stored_name,
           relative_path, mime_type, size_bytes, description, related_entity_type,
           related_entity_id, uploaded_by_user_id, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&file_id)
    .bind(&ctx.clinic_id)
    .bind(&input.patient_id)
    .bind(&category_id)
    .bind(&file_type)
    .bind(input.original_name.trim())
    .bind(&stored_name)
    .bind(&relative_path)
    .bind(input.mime_type.as_deref())
    .bind(input.bytes.len() as i64)
    .bind(input.description.as_deref().map(str::trim))
    .bind(
        input
            .related_entity_type
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    )
    .bind(
        input
            .related_entity_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    )
    .bind(&ctx.user_id)
    .bind(&now)
    .execute(&state.db)
    .await?;

    log_action(
        &state.db,
        Some(&ctx.clinic_id),
        Some(&ctx.user_id),
        "files.create",
        "files",
        Some(&file_id),
        "info",
        Some(json!({
            "patientId": input.patient_id,
            "category": input.category_name,
            "relatedEntityType": input.related_entity_type,
            "relatedEntityId": input.related_entity_id,
            "sizeBytes": input.bytes.len()
        })),
    )
    .await?;

    get_file_by_id(&state.db, &ctx.clinic_id, &file_id).await
}

fn is_allowed_clinical_file(mime_type: Option<&str>, original_name: &str) -> bool {
    let extension = original_name
        .rsplit('.')
        .next()
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    let extension_allowed = matches!(
        extension.as_str(),
        "pdf" | "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tif" | "tiff" | "csv" | "xlsx"
    );
    let mime_allowed = mime_type
        .map(|mime| {
            matches!(
                mime,
                "application/pdf"
                    | "image/png"
                    | "image/jpeg"
                    | "image/webp"
                    | "image/bmp"
                    | "image/tiff"
                    | "text/csv"
                    | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            )
        })
        .unwrap_or(true);
    extension_allowed && mime_allowed
}

pub async fn list_patient_files(
    db: &SqlitePool,
    session_token: &str,
) -> AppResult<Vec<PatientFileSummary>> {
    let ctx = validate_session(db, session_token, Some("patients.view")).await?;
    let rows = sqlx::query_as::<_, PatientFileSummary>(
        r#"
        SELECT f.id, f.patient_id, p.full_name AS patient_name, fc.name AS category_name,
               f.file_type, f.original_name, f.relative_path, f.mime_type, f.size_bytes,
               f.description, f.created_at
        FROM files f
        LEFT JOIN patients p ON p.id = f.patient_id
        LEFT JOIN file_categories fc ON fc.id = f.category_id
        WHERE f.clinic_id = ? AND f.deleted_at IS NULL
        ORDER BY f.created_at DESC
        "#,
    )
    .bind(ctx.clinic_id)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn open_patient_file(
    state: &AppState,
    session_token: &str,
    file_id: &str,
) -> AppResult<()> {
    let ctx = validate_session(&state.db, session_token, Some("patients.view")).await?;
    if file_id.trim().is_empty() {
        return Err(AppError::Validation(
            "Selecciona un archivo para abrir".to_string(),
        ));
    }

    let relative_path: String = sqlx::query_scalar(
        "SELECT relative_path FROM files WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL",
    )
    .bind(&ctx.clinic_id)
    .bind(file_id.trim())
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Validation("Archivo no encontrado".to_string()))?;

    let absolute_path = state.files_dir.join(relative_path);
    let canonical_base = state.files_dir.canonicalize()?;
    let canonical_path = absolute_path.canonicalize()?;
    if !canonical_path.starts_with(canonical_base) {
        return Err(AppError::Conflict(
            "Ruta de archivo fuera del expediente local".to_string(),
        ));
    }

    tauri_plugin_opener::open_path(&canonical_path, None::<&str>)
        .map_err(|error| AppError::Validation(format!("No se pudo abrir el archivo: {error}")))?;
    Ok(())
}

pub async fn open_external_url(db: &SqlitePool, session_token: &str, url: &str) -> AppResult<()> {
    let _ctx = validate_session(db, session_token, None).await?;
    let clean_url = url.trim();
    let allowed = is_allowed_external_url(clean_url);
    if !allowed {
        return Err(AppError::Validation(
            "Enlace externo no permitido".to_string(),
        ));
    }
    tauri_plugin_opener::open_url(clean_url, None::<&str>)
        .map_err(|error| AppError::Validation(format!("No se pudo abrir el enlace: {error}")))?;
    Ok(())
}

fn is_allowed_external_url(url: &str) -> bool {
    if url.is_empty()
        || url.contains(char::is_whitespace)
        || url.contains('\\')
        || url.contains('%')
    {
        return false;
    }
    if let Some(address) = url.strip_prefix("mailto:") {
        return address.contains('@') && !address.contains('/') && !address.contains(':');
    }
    if let Some(value) = url.strip_prefix("https://wa.me/") {
        return value
            .split('?')
            .next()
            .map(|phone| phone.chars().all(|ch| ch.is_ascii_digit()) && phone.len() >= 8)
            .unwrap_or(false);
    }
    false
}

pub async fn list_consent_templates(
    db: &SqlitePool,
    session_token: &str,
) -> AppResult<Vec<ConsentTemplateSummary>> {
    let ctx = validate_session(db, session_token, None).await?;
    let rows = sqlx::query_as::<_, ConsentTemplateSummary>(
        "SELECT id, name, treatment_category, body, active FROM consent_templates WHERE (clinic_id = ? OR clinic_id IS NULL) AND active = 1 ORDER BY name",
    )
    .bind(ctx.clinic_id)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn create_consent_template(
    db: &SqlitePool,
    session_token: &str,
    input: CreateConsentTemplateInput,
) -> AppResult<ConsentTemplateSummary> {
    let ctx = validate_session_for_intent(db, session_token, None, AccessIntent::DataWrite).await?;
    if input.name.trim().is_empty() || input.body.trim().is_empty() {
        return Err(AppError::Validation(
            "Nombre y cuerpo son obligatorios".to_string(),
        ));
    }
    let id = new_id();
    let now = now_utc();
    sqlx::query(
        "INSERT INTO consent_templates (id, clinic_id, name, treatment_category, body, active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 1, ?, ?)",
    )
    .bind(&id)
    .bind(&ctx.clinic_id)
    .bind(input.name.trim())
    .bind(input.treatment_category.as_deref().map(str::trim))
    .bind(input.body.trim())
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;
    get_consent_template_by_id(db, &ctx.clinic_id, &id).await
}

pub async fn get_reports_summary(
    db: &SqlitePool,
    session_token: &str,
    input: ReportsFilterInput,
) -> AppResult<ReportsSummary> {
    let ctx = validate_session(db, session_token, Some("reports.financial")).await?;
    reconcile_system_alerts(db, &ctx.clinic_id).await?;
    let from = input.date_from;
    let to = format!("{}T23:59:59", input.date_to);
    let income_cents = scalar_sum(db, "SELECT COALESCE(SUM(amount_cents), 0) FROM payments WHERE clinic_id = ? AND status = 'active' AND paid_at BETWEEN ? AND ?", &ctx.clinic_id, &from, &to).await?;
    let payments_count = scalar_count(
        db,
        "SELECT COUNT(*) FROM payments WHERE clinic_id = ? AND paid_at BETWEEN ? AND ?",
        &ctx.clinic_id,
        &from,
        &to,
    )
    .await?;
    let appointments_count = scalar_count(
        db,
        "SELECT COUNT(*) FROM appointments WHERE clinic_id = ? AND deleted_at IS NULL AND starts_at BETWEEN ? AND ?",
        &ctx.clinic_id,
        &from,
        &to,
    )
    .await?;
    let cancelled_appointments = scalar_count(db, "SELECT COUNT(*) FROM appointments WHERE clinic_id = ? AND deleted_at IS NULL AND status = 'cancelada' AND starts_at BETWEEN ? AND ?", &ctx.clinic_id, &from, &to).await?;
    let new_patients = scalar_count(db, "SELECT COUNT(*) FROM patients WHERE clinic_id = ? AND created_at BETWEEN ? AND ? AND deleted_at IS NULL", &ctx.clinic_id, &from, &to).await?;
    let estimates_total = scalar_count(
        db,
        "SELECT COUNT(*) FROM estimates WHERE clinic_id = ? AND created_at BETWEEN ? AND ?",
        &ctx.clinic_id,
        &from,
        &to,
    )
    .await?;
    let estimates_approved = scalar_count(db, "SELECT COUNT(*) FROM estimates WHERE clinic_id = ? AND status = 'approved' AND created_at BETWEEN ? AND ?", &ctx.clinic_id, &from, &to).await?;
    let low_inventory: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM inventory_items WHERE clinic_id = ? AND active = 1 AND deleted_at IS NULL AND current_quantity <= minimum_stock")
        .bind(&ctx.clinic_id)
        .fetch_one(db)
        .await?;
    let pending_balances_cents: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(tp.total_cents), 0) - COALESCE(SUM(paid.amount_cents), 0)
        FROM treatment_plans tp
        LEFT JOIN (
          SELECT tpi.treatment_plan_id, SUM(pa.amount_cents) AS amount_cents
          FROM payment_allocations pa
          JOIN treatment_plan_items tpi ON tpi.id = pa.treatment_plan_item_id
          GROUP BY tpi.treatment_plan_id
        ) paid ON paid.treatment_plan_id = tp.id
        WHERE tp.clinic_id = ? AND tp.status NOT IN ('cancelled', 'completed')
        "#,
    )
    .bind(&ctx.clinic_id)
    .fetch_one(db)
    .await?;
    let restock_items = get_restock_items(db, &ctx.clinic_id).await?;
    let income_by_method = sqlx::query_as::<_, ChartPoint>(
        r#"
        SELECT method AS label, COALESCE(SUM(amount_cents), 0) AS value
        FROM payments
        WHERE clinic_id = ? AND status = 'active' AND paid_at BETWEEN ? AND ?
        GROUP BY method
        ORDER BY value DESC
        "#,
    )
    .bind(&ctx.clinic_id)
    .bind(&from)
    .bind(&to)
    .fetch_all(db)
    .await?;
    let appointments_by_status = sqlx::query_as::<_, ChartPoint>(
        r#"
        SELECT status AS label, COUNT(*) AS value
        FROM appointments
        WHERE clinic_id = ? AND deleted_at IS NULL AND starts_at BETWEEN ? AND ?
        GROUP BY status
        ORDER BY value DESC
        "#,
    )
    .bind(&ctx.clinic_id)
    .bind(&from)
    .bind(&to)
    .fetch_all(db)
    .await?;

    Ok(ReportsSummary {
        income_cents,
        payments_count,
        appointments_count,
        cancelled_appointments,
        new_patients,
        estimates_total,
        estimates_approved,
        pending_balances_cents: pending_balances_cents.max(0),
        low_inventory,
        restock_items,
        income_by_method,
        appointments_by_status,
    })
}

pub async fn update_clinic_settings(
    db: &SqlitePool,
    session_token: &str,
    input: UpdateClinicSettingsInput,
) -> AppResult<ClinicSummary> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("settings.edit"),
        AccessIntent::DataWrite,
    )
    .await?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation(
            "El nombre del consultorio es obligatorio".to_string(),
        ));
    }
    let now = now_utc();
    sqlx::query(
        "UPDATE clinics SET name = ?, phone = ?, whatsapp = ?, email = ?, address = ?, tax_data = ?, updated_at = ? WHERE id = ?",
    )
    .bind(input.name.trim())
    .bind(input.phone.as_deref().map(str::trim))
    .bind(input.whatsapp.as_deref().map(str::trim))
    .bind(input.email.as_deref().map(str::trim))
    .bind(input.address.as_deref().map(str::trim))
    .bind(input.tax_data.as_deref().map(str::trim))
    .bind(&now)
    .bind(&ctx.clinic_id)
    .execute(db)
    .await?;
    get_clinic_by_id(db, &ctx.clinic_id).await
}

pub async fn list_message_templates(
    db: &SqlitePool,
    session_token: &str,
) -> AppResult<Vec<MessageTemplateSummary>> {
    let ctx = validate_session(db, session_token, None).await?;
    let rows = sqlx::query_as::<_, MessageTemplateSummary>(
        "SELECT id, name, body FROM message_templates WHERE clinic_id = ? OR clinic_id IS NULL ORDER BY name",
    )
    .bind(ctx.clinic_id)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn global_search(
    db: &SqlitePool,
    session_token: &str,
    term: &str,
) -> AppResult<Vec<GlobalSearchResult>> {
    let ctx = validate_session(db, session_token, None).await?;
    let trimmed = term.trim();
    if trimmed.len() < 2 {
        return Ok(Vec::new());
    }

    let search = normalize_search(trimmed);
    let mut results = Vec::new();

    let mut patients = sqlx::query_as::<_, GlobalSearchResult>(
        r#"
        SELECT 'patient' AS entity_type,
               id,
               full_name AS title,
               COALESCE(NULLIF(phone, ''), NULLIF(whatsapp, ''), NULLIF(email, ''), 'Sin contacto') AS subtitle,
               '/patients' AS route,
               status
        FROM patients
        WHERE clinic_id = ?
          AND deleted_at IS NULL
          AND (full_name LIKE ? OR IFNULL(phone, '') LIKE ? OR IFNULL(whatsapp, '') LIKE ? OR IFNULL(email, '') LIKE ?)
        ORDER BY updated_at DESC
        LIMIT 6
        "#,
    )
    .bind(&ctx.clinic_id)
    .bind(&search)
    .bind(&search)
    .bind(&search)
    .bind(&search)
    .fetch_all(db)
    .await?;
    results.append(&mut patients);

    let mut appointments = sqlx::query_as::<_, GlobalSearchResult>(
        r#"
        SELECT 'appointment' AS entity_type,
               a.id,
               p.full_name AS title,
               a.starts_at || ' - ' || a.reason AS subtitle,
               '/appointments?date=' || substr(a.starts_at, 1, 10) AS route,
               a.status
        FROM appointments a
        JOIN patients p ON p.id = a.patient_id
        WHERE a.clinic_id = ?
          AND a.deleted_at IS NULL
          AND (p.full_name LIKE ? OR a.reason LIKE ? OR a.starts_at LIKE ? OR a.status LIKE ?)
        ORDER BY a.starts_at DESC
        LIMIT 6
        "#,
    )
    .bind(&ctx.clinic_id)
    .bind(&search)
    .bind(&search)
    .bind(&search)
    .bind(&search)
    .fetch_all(db)
    .await?;
    results.append(&mut appointments);

    let mut inventory = sqlx::query_as::<_, GlobalSearchResult>(
        r#"
        SELECT 'inventory' AS entity_type,
               id,
               name AS title,
               category || ' - ' || current_quantity || ' ' || unit AS subtitle,
               '/inventory' AS route,
               CASE WHEN current_quantity <= minimum_stock THEN 'bajo' ELSE 'activo' END AS status
        FROM inventory_items
        WHERE clinic_id = ?
          AND active = 1
          AND deleted_at IS NULL
          AND (name LIKE ? OR category LIKE ? OR IFNULL(location, '') LIKE ?)
        ORDER BY updated_at DESC
        LIMIT 6
        "#,
    )
    .bind(&ctx.clinic_id)
    .bind(&search)
    .bind(&search)
    .bind(&search)
    .fetch_all(db)
    .await?;
    results.append(&mut inventory);

    let mut estimates = sqlx::query_as::<_, GlobalSearchResult>(
        r#"
        SELECT 'estimate' AS entity_type,
               e.id,
               e.folio AS title,
               p.full_name || ' - ' || printf('%.2f', e.total_cents / 100.0) AS subtitle,
               '/estimates' AS route,
               e.status
        FROM estimates e
        JOIN patients p ON p.id = e.patient_id
        WHERE e.clinic_id = ?
          AND (e.folio LIKE ? OR p.full_name LIKE ? OR e.status LIKE ?)
        ORDER BY e.created_at DESC
        LIMIT 6
        "#,
    )
    .bind(&ctx.clinic_id)
    .bind(&search)
    .bind(&search)
    .bind(&search)
    .fetch_all(db)
    .await?;
    results.append(&mut estimates);

    results.truncate(18);
    Ok(results)
}

pub async fn list_roles(db: &SqlitePool, session_token: &str) -> AppResult<Vec<RoleSummary>> {
    let ctx = validate_session(db, session_token, Some("users.admin")).await?;
    let rows = sqlx::query_as::<_, RoleSummary>(
        "SELECT id, name, system_key FROM roles WHERE clinic_id = ? ORDER BY name",
    )
    .bind(ctx.clinic_id)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn create_user(
    db: &SqlitePool,
    session_token: &str,
    input: CreateUserInput,
) -> AppResult<UserListItem> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("users.admin"),
        AccessIntent::DataWrite,
    )
    .await?;
    if input.full_name.trim().is_empty()
        || input.username.trim().is_empty()
        || input.role_id.trim().is_empty()
    {
        return Err(AppError::Validation(
            "Nombre, usuario y rol son obligatorios".to_string(),
        ));
    }
    let id = new_id();
    let now = now_utc();
    let password_hash = hash_password(&input.password)?;
    sqlx::query(
        r#"
        INSERT INTO users
          (id, clinic_id, role_id, full_name, username, email, password_hash, status,
           professional_license, specialty, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 'active', ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&ctx.clinic_id)
    .bind(&input.role_id)
    .bind(input.full_name.trim())
    .bind(input.username.trim())
    .bind(
        input
            .email
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    )
    .bind(password_hash)
    .bind(input.professional_license.as_deref().map(str::trim))
    .bind(input.specialty.as_deref().map(str::trim))
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;
    let user = sqlx::query_as::<_, UserListItem>(
        "SELECT u.id, u.full_name, u.username, r.name AS role_name FROM users u LEFT JOIN roles r ON r.id = u.role_id WHERE u.id = ?",
    )
    .bind(&id)
    .fetch_one(db)
    .await?;
    Ok(user)
}

pub async fn list_periodontal_records(
    db: &SqlitePool,
    session_token: &str,
) -> AppResult<Vec<PeriodontalRecordSummary>> {
    let ctx = validate_session(db, session_token, Some("clinical.view")).await?;
    let rows = sqlx::query_as::<_, PeriodontalRecordSummary>(
        r#"
        SELECT pr.id, pr.patient_id, p.full_name AS patient_name, pr.status,
               pr.notes, pr.created_at, pr.updated_at
        FROM periodontal_records pr
        JOIN patients p ON p.id = pr.patient_id
        WHERE pr.clinic_id = ?
        ORDER BY pr.created_at DESC
        "#,
    )
    .bind(ctx.clinic_id)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn create_periodontal_record(
    db: &SqlitePool,
    session_token: &str,
    input: CreatePeriodontalRecordInput,
) -> AppResult<PeriodontalRecordSummary> {
    let ctx = validate_session_for_intent(
        db,
        session_token,
        Some("clinical.edit"),
        AccessIntent::DataWrite,
    )
    .await?;
    if input.patient_id.trim().is_empty() {
        return Err(AppError::Validation("Selecciona un paciente".to_string()));
    }
    let id = new_id();
    let now = now_utc();
    sqlx::query(
        "INSERT INTO periodontal_records (id, clinic_id, patient_id, status, notes, created_at, updated_at) VALUES (?, ?, ?, 'draft', ?, ?, ?)",
    )
    .bind(&id)
    .bind(&ctx.clinic_id)
    .bind(&input.patient_id)
    .bind(input.notes.as_deref().map(str::trim))
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;
    sqlx::query_as::<_, PeriodontalRecordSummary>(
        r#"
        SELECT pr.id, pr.patient_id, p.full_name AS patient_name, pr.status,
               pr.notes, pr.created_at, pr.updated_at
        FROM periodontal_records pr
        JOIN patients p ON p.id = pr.patient_id
        WHERE pr.clinic_id = ? AND pr.id = ?
        "#,
    )
    .bind(&ctx.clinic_id)
    .bind(&id)
    .fetch_one(db)
    .await
    .map_err(Into::into)
}

pub async fn refresh_system_alerts(db: &SqlitePool, clinic_id: &str) -> AppResult<()> {
    let now = now_utc();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let soon = (chrono::Local::now().date_naive() + chrono::Duration::days(30))
        .format("%Y-%m-%d")
        .to_string();

    let low_items = sqlx::query_as::<_, (String, String, f64, f64, Option<String>)>(
        r#"
        SELECT id, name, current_quantity, minimum_stock, expiration_date
        FROM inventory_items
        WHERE clinic_id = ? AND active = 1 AND deleted_at IS NULL AND current_quantity <= minimum_stock
        "#,
    )
    .bind(clinic_id)
    .fetch_all(db)
    .await?;
    for (id, name, current, minimum, _) in low_items {
        insert_system_alert_if_missing(
            db,
            clinic_id,
            "inventario_bajo",
            "inventory_items",
            &id,
            "alta",
            "Inventario bajo",
            &format!("{name}: existencia {current}, mínimo {minimum}."),
            None,
            &now,
        )
        .await?;
    }

    let expiring_items = sqlx::query_as::<_, (String, String, String)>(
        r#"
        SELECT id, name, expiration_date
        FROM inventory_items
        WHERE clinic_id = ?
          AND active = 1
          AND deleted_at IS NULL
          AND expiration_date IS NOT NULL
          AND expiration_date != ''
          AND expiration_date <= ?
        "#,
    )
    .bind(clinic_id)
    .bind(&soon)
    .fetch_all(db)
    .await?;
    for (id, name, expiration_date) in expiring_items {
        let (priority, title) = if expiration_date <= today {
            ("critica", "Insumo caducado")
        } else {
            ("media", "Insumo próximo a caducar")
        };
        insert_system_alert_if_missing(
            db,
            clinic_id,
            if expiration_date <= today {
                "insumo_caducado"
            } else {
                "insumo_por_caducar"
            },
            "inventory_items",
            &id,
            priority,
            title,
            &format!("{name} vence el {expiration_date}."),
            Some(&expiration_date),
            &now,
        )
        .await?;
    }

    let pending_appointments = sqlx::query_as::<_, (String, String, String)>(
        r#"
        SELECT a.id, p.full_name, a.starts_at
        FROM appointments a
        JOIN patients p ON p.id = a.patient_id
        WHERE a.clinic_id = ?
          AND a.deleted_at IS NULL
          AND a.status = 'programada'
          AND a.starts_at >= ?
          AND a.starts_at < date(?, '+2 day')
        "#,
    )
    .bind(clinic_id)
    .bind(&today)
    .bind(&today)
    .fetch_all(db)
    .await?;
    for (id, patient_name, starts_at) in pending_appointments {
        insert_system_alert_if_missing(
            db,
            clinic_id,
            "cita_sin_confirmar",
            "appointments",
            &id,
            "media",
            "Cita sin confirmar",
            &format!("{patient_name} tiene cita programada el {starts_at} sin confirmar."),
            Some(&starts_at),
            &now,
        )
        .await?;
    }

    let expired_estimates = sqlx::query_as::<_, (String, String, String, String)>(
        r#"
        SELECT e.id, e.folio, p.full_name, e.valid_until
        FROM estimates e
        JOIN patients p ON p.id = e.patient_id
        WHERE e.clinic_id = ?
          AND e.status IN ('draft', 'delivered')
          AND e.valid_until IS NOT NULL
          AND e.valid_until != ''
          AND e.valid_until < ?
        "#,
    )
    .bind(clinic_id)
    .bind(&today)
    .fetch_all(db)
    .await?;
    for (id, folio, patient_name, valid_until) in expired_estimates {
        insert_system_alert_if_missing(
            db,
            clinic_id,
            "presupuesto_vencido",
            "estimates",
            &id,
            "media",
            "Presupuesto vencido",
            &format!("El presupuesto {folio} de {patient_name} venció el {valid_until}."),
            Some(&valid_until),
            &now,
        )
        .await?;
    }

    let last_backup: Option<String> = sqlx::query_scalar(
        "SELECT MAX(created_at) FROM backups WHERE clinic_id = ? AND status = 'completed'",
    )
    .bind(clinic_id)
    .fetch_optional(db)
    .await?;
    let backup_is_stale = last_backup
        .as_deref()
        .map(|value| &value[..value.len().min(10)] < today.as_str())
        .unwrap_or(true);
    if backup_is_stale {
        insert_system_alert_if_missing(
            db,
            clinic_id,
            "respaldo_pendiente",
            "system",
            "backups",
            "alta",
            "Respaldo pendiente",
            "No hay un respaldo completado hoy.",
            None,
            &now,
        )
        .await?;
    }

    Ok(())
}

pub async fn reconcile_system_alerts(db: &SqlitePool, clinic_id: &str) -> AppResult<()> {
    let now = now_utc();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    sqlx::query(
        r#"
        UPDATE alerts
        SET status = 'resolved', resolved_at = ?
        WHERE clinic_id = ?
          AND status = 'open'
          AND alert_type = 'inventario_bajo'
          AND related_entity_type = 'inventory_items'
          AND NOT EXISTS (
            SELECT 1
            FROM inventory_items ii
            WHERE ii.id = alerts.related_entity_id
              AND ii.clinic_id = alerts.clinic_id
              AND ii.active = 1
              AND ii.deleted_at IS NULL
              AND ii.current_quantity <= ii.minimum_stock
          )
        "#,
    )
    .bind(&now)
    .bind(clinic_id)
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        UPDATE alerts
        SET status = 'resolved', resolved_at = ?
        WHERE clinic_id = ?
          AND status = 'open'
          AND alert_type IN ('insumo_caducado', 'insumo_por_caducar')
          AND related_entity_type = 'inventory_items'
          AND NOT EXISTS (
            SELECT 1
            FROM inventory_items ii
            WHERE ii.id = alerts.related_entity_id
              AND ii.clinic_id = alerts.clinic_id
              AND ii.active = 1
              AND ii.deleted_at IS NULL
              AND ii.expiration_date IS NOT NULL
              AND ii.expiration_date != ''
              AND ii.expiration_date <= date('now', '+30 day')
          )
        "#,
    )
    .bind(&now)
    .bind(clinic_id)
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        UPDATE alerts
        SET status = 'resolved', resolved_at = ?
        WHERE clinic_id = ?
          AND status = 'open'
          AND alert_type = 'cita_sin_confirmar'
          AND related_entity_type = 'appointments'
          AND NOT EXISTS (
            SELECT 1
            FROM appointments a
            WHERE a.id = alerts.related_entity_id
              AND a.clinic_id = alerts.clinic_id
              AND a.deleted_at IS NULL
              AND a.status = 'programada'
              AND a.starts_at >= date('now')
              AND a.starts_at < date('now', '+2 day')
          )
        "#,
    )
    .bind(&now)
    .bind(clinic_id)
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        UPDATE alerts
        SET status = 'resolved', resolved_at = ?
        WHERE clinic_id = ?
          AND status = 'open'
          AND alert_type = 'presupuesto_vencido'
          AND related_entity_type = 'estimates'
          AND NOT EXISTS (
            SELECT 1
            FROM estimates e
            WHERE e.id = alerts.related_entity_id
              AND e.clinic_id = alerts.clinic_id
              AND e.status IN ('draft', 'delivered')
              AND e.valid_until IS NOT NULL
              AND e.valid_until != ''
              AND e.valid_until < date('now')
          )
        "#,
    )
    .bind(&now)
    .bind(clinic_id)
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        UPDATE alerts
        SET status = 'resolved', resolved_at = ?
        WHERE clinic_id = ?
          AND status = 'open'
          AND alert_type = 'respaldo_pendiente'
          AND related_entity_type = 'system'
          AND related_entity_id = 'backups'
          AND EXISTS (
            SELECT 1
            FROM backups b
            WHERE b.clinic_id = alerts.clinic_id
              AND b.status = 'completed'
              AND substr(b.created_at, 1, 10) = ?
          )
        "#,
    )
    .bind(&now)
    .bind(clinic_id)
    .bind(&today)
    .execute(db)
    .await?;

    refresh_system_alerts(db, clinic_id).await
}

#[expect(
    clippy::too_many_arguments,
    reason = "System alert deduplication needs the alert identity and display fields together."
)]
async fn insert_system_alert_if_missing(
    db: &SqlitePool,
    clinic_id: &str,
    alert_type: &str,
    related_entity_type: &str,
    related_entity_id: &str,
    priority: &str,
    title: &str,
    message: &str,
    due_at: Option<&str>,
    now: &str,
) -> AppResult<()> {
    let existing: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM alerts
        WHERE clinic_id = ?
          AND alert_type = ?
          AND related_entity_type = ?
          AND related_entity_id = ?
          AND status = 'open'
        "#,
    )
    .bind(clinic_id)
    .bind(alert_type)
    .bind(related_entity_type)
    .bind(related_entity_id)
    .fetch_one(db)
    .await?;
    if existing > 0 {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO alerts
          (id, clinic_id, alert_type, priority, title, message, due_at, status,
           related_entity_type, related_entity_id, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 'open', ?, ?, ?)
        "#,
    )
    .bind(new_id())
    .bind(clinic_id)
    .bind(alert_type)
    .bind(priority)
    .bind(title)
    .bind(message)
    .bind(due_at)
    .bind(related_entity_type)
    .bind(related_entity_id)
    .bind(now)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn get_restock_items(
    db: &SqlitePool,
    clinic_id: &str,
) -> AppResult<Vec<RestockReportItem>> {
    let rows = sqlx::query_as::<_, RestockReportItem>(
        r#"
        SELECT ii.id, ii.name, ii.category, ii.unit, ii.current_quantity, ii.minimum_stock,
               CASE
                 WHEN ii.current_quantity < ii.minimum_stock
                 THEN (ii.minimum_stock - ii.current_quantity)
                 ELSE 0
               END AS suggested_quantity,
               ii.cost_cents,
               CAST(ROUND(CASE
                 WHEN ii.current_quantity < ii.minimum_stock
                 THEN (ii.minimum_stock - ii.current_quantity) * ii.cost_cents
                 ELSE 0
               END) AS INTEGER) AS estimated_cost_cents,
               s.name AS supplier_name,
               ii.expiration_date,
               ii.location
        FROM inventory_items ii
        LEFT JOIN suppliers s ON s.id = ii.supplier_id
        WHERE ii.clinic_id = ?
          AND ii.active = 1
          AND ii.deleted_at IS NULL
          AND (ii.current_quantity <= ii.minimum_stock
               OR (ii.expiration_date IS NOT NULL
                   AND ii.expiration_date != ''
                   AND ii.expiration_date <= date('now', '+30 day')))
        ORDER BY
          CASE WHEN ii.current_quantity <= ii.minimum_stock THEN 0 ELSE 1 END,
          ii.category,
          ii.name
        "#,
    )
    .bind(clinic_id)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

async fn ensure_unique_treatment_name(
    db: &SqlitePool,
    clinic_id: &str,
    name: &str,
    category: &str,
    except_id: Option<&str>,
) -> AppResult<()> {
    let count: i64 = if let Some(except_id) = except_id {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM treatment_catalog
            WHERE active = 1
              AND (clinic_id = ? OR clinic_id IS NULL)
              AND LOWER(name) = LOWER(?)
              AND LOWER(category) = LOWER(?)
              AND id <> ?
            "#,
        )
        .bind(clinic_id)
        .bind(name)
        .bind(category)
        .bind(except_id)
        .fetch_one(db)
        .await?
    } else {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM treatment_catalog
            WHERE active = 1
              AND (clinic_id = ? OR clinic_id IS NULL)
              AND LOWER(name) = LOWER(?)
              AND LOWER(category) = LOWER(?)
            "#,
        )
        .bind(clinic_id)
        .bind(name)
        .bind(category)
        .fetch_one(db)
        .await?
    };

    if count > 0 {
        return Err(AppError::Validation(
            "Ese tratamiento ya existe en el catálogo. Edítalo para evitar duplicados.".to_string(),
        ));
    }
    Ok(())
}

async fn ensure_unique_inventory_item(
    db: &SqlitePool,
    clinic_id: &str,
    name: &str,
    category: &str,
    unit: &str,
    except_id: Option<&str>,
) -> AppResult<()> {
    let count: i64 = if let Some(except_id) = except_id {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM inventory_items
            WHERE clinic_id = ?
              AND active = 1
              AND deleted_at IS NULL
              AND LOWER(name) = LOWER(?)
              AND LOWER(category) = LOWER(?)
              AND LOWER(unit) = LOWER(?)
              AND id <> ?
            "#,
        )
        .bind(clinic_id)
        .bind(name)
        .bind(category)
        .bind(unit)
        .bind(except_id)
        .fetch_one(db)
        .await?
    } else {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM inventory_items
            WHERE clinic_id = ?
              AND active = 1
              AND deleted_at IS NULL
              AND LOWER(name) = LOWER(?)
              AND LOWER(category) = LOWER(?)
              AND LOWER(unit) = LOWER(?)
            "#,
        )
        .bind(clinic_id)
        .bind(name)
        .bind(category)
        .bind(unit)
        .fetch_one(db)
        .await?
    };

    if count > 0 {
        return Err(AppError::Validation(
            "Ya existe un insumo activo con el mismo nombre, categoría y unidad".to_string(),
        ));
    }
    Ok(())
}

async fn next_folio(db: &SqlitePool, clinic_id: &str, entity: &str) -> AppResult<String> {
    let mut tx = db.begin().await?;
    let row: Option<(String, i64)> = sqlx::query_as(
        "SELECT prefix, next_number FROM folio_sequences WHERE clinic_id = ? AND entity = ?",
    )
    .bind(clinic_id)
    .bind(entity)
    .fetch_optional(&mut *tx)
    .await?;
    let existed = row.is_some();
    let (prefix, next_number) = row.unwrap_or_else(|| (entity.to_uppercase(), 1));
    if !existed {
        sqlx::query(
            "INSERT INTO folio_sequences (id, clinic_id, entity, prefix, next_number, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(new_id())
        .bind(clinic_id)
        .bind(entity)
        .bind(&prefix)
        .bind(next_number + 1)
        .bind(now_utc())
        .execute(&mut *tx)
        .await?;
    } else {
        sqlx::query("UPDATE folio_sequences SET next_number = next_number + 1, updated_at = ? WHERE clinic_id = ? AND entity = ?")
            .bind(now_utc())
            .bind(clinic_id)
            .bind(entity)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(format!("{}-{:06}", prefix, next_number))
}

async fn get_treatment_by_id(db: &SqlitePool, id: &str) -> AppResult<TreatmentCatalogItem> {
    sqlx::query_as::<_, TreatmentCatalogItem>(
        r#"
        SELECT id, name, category, description, base_price_cents, estimated_duration_minutes,
               requires_follow_up, active
        FROM treatment_catalog WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_one(db)
    .await
    .map_err(Into::into)
}

async fn get_treatment_plan_by_id(
    db: &SqlitePool,
    clinic_id: &str,
    id: &str,
) -> AppResult<TreatmentPlanSummary> {
    sqlx::query_as::<_, TreatmentPlanSummary>(
        r#"
        SELECT tp.id, tp.patient_id, p.full_name AS patient_name, tp.diagnosis, tp.subtotal_cents,
               tp.discount_cents, tp.total_cents,
               COALESCE((SELECT SUM(pa.amount_cents)
                         FROM payment_allocations pa
                         JOIN treatment_plan_items tpi ON tpi.id = pa.treatment_plan_item_id
                         WHERE tpi.treatment_plan_id = tp.id), 0) AS paid_cents,
               (tp.total_cents - COALESCE((SELECT SUM(pa.amount_cents)
                         FROM payment_allocations pa
                         JOIN treatment_plan_items tpi ON tpi.id = pa.treatment_plan_item_id
                         WHERE tpi.treatment_plan_id = tp.id), 0)) AS balance_cents,
               tp.status, tp.notes, tp.created_at
        FROM treatment_plans tp
        JOIN patients p ON p.id = tp.patient_id
        WHERE tp.clinic_id = ? AND tp.id = ?
        "#,
    )
    .bind(clinic_id)
    .bind(id)
    .fetch_one(db)
    .await
    .map_err(Into::into)
}

async fn get_estimate_by_id(
    db: &SqlitePool,
    clinic_id: &str,
    id: &str,
) -> AppResult<EstimateSummary> {
    sqlx::query_as::<_, EstimateSummary>(
        r#"
        SELECT e.id, e.patient_id, p.full_name AS patient_name, e.treatment_plan_id, e.folio,
               e.status, e.valid_until, e.subtotal_cents, e.discount_cents, e.total_cents,
               e.observations, e.terms, e.created_at
        FROM estimates e
        JOIN patients p ON p.id = e.patient_id
        WHERE e.clinic_id = ? AND e.id = ?
        "#,
    )
    .bind(clinic_id)
    .bind(id)
    .fetch_one(db)
    .await
    .map_err(Into::into)
}

async fn get_payment_by_id(
    db: &SqlitePool,
    clinic_id: &str,
    id: &str,
) -> AppResult<PaymentSummary> {
    sqlx::query_as::<_, PaymentSummary>(
        r#"
        SELECT pmt.id, pmt.patient_id, p.full_name AS patient_name, pmt.folio, pmt.concept,
               pmt.amount_cents, pmt.method, pmt.status, pmt.paid_at,
               u.full_name AS received_by_name, pmt.notes,
               COALESCE((
                 SELECT COUNT(*)
                 FROM files f
                 WHERE f.clinic_id = pmt.clinic_id
                   AND f.related_entity_type = 'payments'
                   AND f.related_entity_id = pmt.id
                   AND f.deleted_at IS NULL
               ), 0) AS proof_files_count
        FROM payments pmt
        JOIN patients p ON p.id = pmt.patient_id
        JOIN users u ON u.id = pmt.received_by_user_id
        WHERE pmt.clinic_id = ? AND pmt.id = ?
        "#,
    )
    .bind(clinic_id)
    .bind(id)
    .fetch_one(db)
    .await
    .map_err(Into::into)
}

async fn current_cash_register_id_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    clinic_id: &str,
) -> AppResult<Option<String>> {
    let id = sqlx::query_scalar("SELECT id FROM cash_registers WHERE clinic_id = ? AND status = 'open' ORDER BY opened_at DESC LIMIT 1")
        .bind(clinic_id)
        .fetch_optional(&mut **tx)
        .await?;
    Ok(id)
}

async fn get_cash_register_by_id(
    db: &SqlitePool,
    clinic_id: &str,
    id: &str,
) -> AppResult<CashRegisterSummary> {
    sqlx::query_as::<_, CashRegisterSummary>(
        r#"
        SELECT cr.id, u.full_name AS opened_by_name, cr.opened_at, cr.opening_float_cents,
               cr.status, cr.closed_at,
               COALESCE(SUM(CASE WHEN cm.method = 'efectivo' AND cm.cancelled_at IS NULL THEN cm.amount_cents ELSE 0 END), 0) AS total_cash_cents,
               COALESCE(SUM(CASE WHEN cm.method = 'transferencia' AND cm.cancelled_at IS NULL THEN cm.amount_cents ELSE 0 END), 0) AS total_transfer_cents,
               COALESCE(SUM(CASE WHEN cm.method = 'tarjeta' AND cm.cancelled_at IS NULL THEN cm.amount_cents ELSE 0 END), 0) AS total_card_cents,
               COALESCE(SUM(CASE WHEN cm.method NOT IN ('efectivo', 'transferencia', 'tarjeta') AND cm.cancelled_at IS NULL THEN cm.amount_cents ELSE 0 END), 0) AS total_other_cents
        FROM cash_registers cr
        JOIN users u ON u.id = cr.opened_by_user_id
        LEFT JOIN cash_movements cm ON cm.cash_register_id = cr.id
        WHERE cr.clinic_id = ? AND cr.id = ?
        GROUP BY cr.id
        "#,
    )
    .bind(clinic_id)
    .bind(id)
    .fetch_one(db)
    .await
    .map_err(Into::into)
}

async fn get_supplier_by_id(
    db: &SqlitePool,
    clinic_id: &str,
    id: &str,
) -> AppResult<SupplierSummary> {
    sqlx::query_as::<_, SupplierSummary>(
        "SELECT id, name, phone, email, notes, active FROM suppliers WHERE clinic_id = ? AND id = ?",
    )
    .bind(clinic_id)
    .bind(id)
    .fetch_one(db)
    .await
    .map_err(Into::into)
}

async fn get_inventory_item_by_id(
    db: &SqlitePool,
    clinic_id: &str,
    id: &str,
) -> AppResult<InventoryItemSummary> {
    sqlx::query_as::<_, InventoryItemSummary>(
        r#"
        SELECT ii.id, ii.supplier_id, s.name AS supplier_name, ii.name, ii.category, ii.unit,
               ii.current_quantity, ii.minimum_stock, ii.cost_cents, ii.expiration_date,
               ii.location, ii.active
        FROM inventory_items ii
        LEFT JOIN suppliers s ON s.id = ii.supplier_id
        WHERE ii.clinic_id = ? AND ii.id = ? AND ii.deleted_at IS NULL
        "#,
    )
    .bind(clinic_id)
    .bind(id)
    .fetch_one(db)
    .await
    .map_err(Into::into)
}

async fn get_alert_by_id(db: &SqlitePool, clinic_id: &str, id: &str) -> AppResult<AlertSummary> {
    sqlx::query_as::<_, AlertSummary>(
        r#"
        SELECT a.id, p.full_name AS patient_name, a.alert_type, a.priority, a.title,
               a.message, a.due_at, a.status, a.created_at
        FROM alerts a
        LEFT JOIN patients p ON p.id = a.patient_id
        WHERE a.clinic_id = ? AND a.id = ?
        "#,
    )
    .bind(clinic_id)
    .bind(id)
    .fetch_one(db)
    .await
    .map_err(Into::into)
}

async fn ensure_file_category(db: &SqlitePool, name: &str) -> AppResult<String> {
    let clean_name = if name.trim().is_empty() {
        "Varios"
    } else {
        name.trim()
    };
    if let Some(id) =
        sqlx::query_scalar::<_, String>("SELECT id FROM file_categories WHERE name = ?")
            .bind(clean_name)
            .fetch_optional(db)
            .await?
    {
        return Ok(id);
    }
    let id = new_id();
    sqlx::query("INSERT INTO file_categories (id, name, description) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(clean_name)
        .bind("Categoría creada por el usuario")
        .execute(db)
        .await?;
    Ok(id)
}

async fn get_file_by_id(
    db: &SqlitePool,
    clinic_id: &str,
    id: &str,
) -> AppResult<PatientFileSummary> {
    sqlx::query_as::<_, PatientFileSummary>(
        r#"
        SELECT f.id, f.patient_id, p.full_name AS patient_name, fc.name AS category_name,
               f.file_type, f.original_name, f.relative_path, f.mime_type, f.size_bytes,
               f.description, f.created_at
        FROM files f
        LEFT JOIN patients p ON p.id = f.patient_id
        LEFT JOIN file_categories fc ON fc.id = f.category_id
        WHERE f.clinic_id = ? AND f.id = ?
        "#,
    )
    .bind(clinic_id)
    .bind(id)
    .fetch_one(db)
    .await
    .map_err(Into::into)
}

async fn get_consent_template_by_id(
    db: &SqlitePool,
    clinic_id: &str,
    id: &str,
) -> AppResult<ConsentTemplateSummary> {
    sqlx::query_as::<_, ConsentTemplateSummary>(
        "SELECT id, name, treatment_category, body, active FROM consent_templates WHERE (clinic_id = ? OR clinic_id IS NULL) AND id = ?",
    )
    .bind(clinic_id)
    .bind(id)
    .fetch_one(db)
    .await
    .map_err(Into::into)
}

async fn get_clinic_by_id(db: &SqlitePool, clinic_id: &str) -> AppResult<ClinicSummary> {
    sqlx::query_as::<_, ClinicSummary>(
        "SELECT id, name, subtitle, phone, whatsapp, email, address FROM clinics WHERE id = ?",
    )
    .bind(clinic_id)
    .fetch_one(db)
    .await
    .map_err(Into::into)
}

async fn scalar_count(
    db: &SqlitePool,
    sql: &str,
    clinic_id: &str,
    from: &str,
    to: &str,
) -> AppResult<i64> {
    sqlx::query_scalar(sql)
        .bind(clinic_id)
        .bind(from)
        .bind(to)
        .fetch_one(db)
        .await
        .map_err(Into::into)
}

async fn scalar_sum(
    db: &SqlitePool,
    sql: &str,
    clinic_id: &str,
    from: &str,
    to: &str,
) -> AppResult<i64> {
    sqlx::query_scalar(sql)
        .bind(clinic_id)
        .bind(from)
        .bind(to)
        .fetch_one(db)
        .await
        .map_err(Into::into)
}
