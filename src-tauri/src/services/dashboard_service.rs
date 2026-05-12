use chrono::{Datelike, Duration, Local};
use sqlx::SqlitePool;

use crate::errors::AppResult;
use crate::models::{AlertSummary, AppointmentSummary, ChartPoint, DashboardSummary};
use crate::services::auth_service::validate_session;
use crate::services::office_service::{get_restock_items, refresh_system_alerts};
use crate::utils::today_prefix;

pub async fn get_dashboard_summary(
    db: &SqlitePool,
    session_token: &str,
) -> AppResult<DashboardSummary> {
    let ctx = validate_session(db, session_token, None).await?;
    refresh_system_alerts(db, &ctx.clinic_id).await?;
    let today = today_prefix();
    let today_like = format!("{today}%");
    let week_start = (Local::now().date_naive() - Duration::days(6))
        .format("%Y-%m-%d")
        .to_string();
    let month_start = format!("{}-{:02}-01", Local::now().year(), Local::now().month());

    let appointments_today = count_scalar(
        db,
        "SELECT COUNT(*) FROM appointments WHERE clinic_id = ? AND starts_at LIKE ?",
        &ctx.clinic_id,
        &today_like,
    )
    .await?;
    let confirmed_today = count_status(db, &ctx.clinic_id, &today_like, "confirmada").await?;
    let unconfirmed_today = count_status(db, &ctx.clinic_id, &today_like, "programada").await?;
    let waiting_today = count_status(db, &ctx.clinic_id, &today_like, "en_espera").await?;

    let revenue_today_cents: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_cents), 0) FROM payments WHERE clinic_id = ? AND status = 'active' AND paid_at LIKE ?",
    )
    .bind(&ctx.clinic_id)
    .bind(&today_like)
    .fetch_one(db)
    .await?;

    let revenue_week_cents: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_cents), 0) FROM payments WHERE clinic_id = ? AND status = 'active' AND paid_at >= ?",
    )
    .bind(&ctx.clinic_id)
    .bind(&week_start)
    .fetch_one(db)
    .await?;

    let revenue_month_cents: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_cents), 0) FROM payments WHERE clinic_id = ? AND status = 'active' AND paid_at >= ?",
    )
    .bind(&ctx.clinic_id)
    .bind(&month_start)
    .fetch_one(db)
    .await?;

    let pending_estimates: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM estimates WHERE clinic_id = ? AND status IN ('draft', 'delivered')",
    )
    .bind(&ctx.clinic_id)
    .fetch_one(db)
    .await?;

    let approved_estimates: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM estimates WHERE clinic_id = ? AND status = 'approved'",
    )
    .bind(&ctx.clinic_id)
    .fetch_one(db)
    .await?;

    let new_patients_month: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM patients WHERE clinic_id = ? AND deleted_at IS NULL AND created_at >= ?",
    )
    .bind(&ctx.clinic_id)
    .bind(&month_start)
    .fetch_one(db)
    .await?;

    let low_inventory: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inventory_items WHERE clinic_id = ? AND active = 1 AND current_quantity <= minimum_stock",
    )
    .bind(&ctx.clinic_id)
    .fetch_one(db)
    .await?;

    let open_alerts: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM alerts WHERE clinic_id = ? AND status = 'open'")
            .bind(&ctx.clinic_id)
            .fetch_one(db)
            .await?;

    let active_treatment_plans: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM treatment_plans WHERE clinic_id = ? AND status IN ('approved', 'in_process')",
    )
    .bind(&ctx.clinic_id)
    .fetch_one(db)
    .await?;

    let upcoming_appointments = sqlx::query_as::<_, AppointmentSummary>(
        r#"
        SELECT a.id, a.patient_id, p.full_name AS patient_name, a.dentist_user_id,
               u.full_name AS dentist_name, a.starts_at, a.ends_at, a.duration_minutes,
               a.reason, a.appointment_type, a.status, a.notes
        FROM appointments a
        JOIN patients p ON p.id = a.patient_id
        LEFT JOIN users u ON u.id = a.dentist_user_id
        WHERE a.clinic_id = ? AND a.starts_at >= ? AND a.status NOT IN ('cancelada', 'finalizada')
        ORDER BY a.starts_at
        LIMIT 6
        "#,
    )
    .bind(&ctx.clinic_id)
    .bind(&today)
    .fetch_all(db)
    .await?;
    let income_series = sqlx::query_as::<_, ChartPoint>(
        r#"
        SELECT substr(paid_at, 1, 10) AS label, COALESCE(SUM(amount_cents), 0) AS value
        FROM payments
        WHERE clinic_id = ? AND status = 'active' AND paid_at >= ?
        GROUP BY substr(paid_at, 1, 10)
        ORDER BY label
        "#,
    )
    .bind(&ctx.clinic_id)
    .bind(&week_start)
    .fetch_all(db)
    .await?;
    let appointment_statuses = sqlx::query_as::<_, ChartPoint>(
        r#"
        SELECT status AS label, COUNT(*) AS value
        FROM appointments
        WHERE clinic_id = ? AND starts_at LIKE ?
        GROUP BY status
        ORDER BY value DESC
        "#,
    )
    .bind(&ctx.clinic_id)
    .bind(&today_like)
    .fetch_all(db)
    .await?;
    let payment_methods = sqlx::query_as::<_, ChartPoint>(
        r#"
        SELECT method AS label, COALESCE(SUM(amount_cents), 0) AS value
        FROM payments
        WHERE clinic_id = ? AND status = 'active' AND paid_at >= ?
        GROUP BY method
        ORDER BY value DESC
        "#,
    )
    .bind(&ctx.clinic_id)
    .bind(&month_start)
    .fetch_all(db)
    .await?;
    let critical_alerts = sqlx::query_as::<_, AlertSummary>(
        r#"
        SELECT a.id, p.full_name AS patient_name, a.alert_type, a.priority, a.title,
               a.message, a.due_at, a.status, a.created_at
        FROM alerts a
        LEFT JOIN patients p ON p.id = a.patient_id
        WHERE a.clinic_id = ? AND a.status = 'open' AND a.priority IN ('critica', 'alta')
        ORDER BY CASE a.priority WHEN 'critica' THEN 0 ELSE 1 END, a.created_at DESC
        LIMIT 6
        "#,
    )
    .bind(&ctx.clinic_id)
    .fetch_all(db)
    .await?;
    let restock_items = get_restock_items(db, &ctx.clinic_id).await?;

    Ok(DashboardSummary {
        appointments_today,
        confirmed_today,
        unconfirmed_today,
        waiting_today,
        revenue_today_cents,
        revenue_week_cents,
        revenue_month_cents,
        pending_estimates,
        approved_estimates,
        new_patients_month,
        low_inventory,
        open_alerts,
        active_treatment_plans,
        upcoming_appointments,
        income_series,
        appointment_statuses,
        payment_methods,
        critical_alerts,
        restock_items,
    })
}

async fn count_status(
    db: &SqlitePool,
    clinic_id: &str,
    day_like: &str,
    status: &str,
) -> AppResult<i64> {
    let value = sqlx::query_scalar(
        "SELECT COUNT(*) FROM appointments WHERE clinic_id = ? AND starts_at LIKE ? AND status = ?",
    )
    .bind(clinic_id)
    .bind(day_like)
    .bind(status)
    .fetch_one(db)
    .await?;
    Ok(value)
}

async fn count_scalar(db: &SqlitePool, sql: &str, clinic_id: &str, value: &str) -> AppResult<i64> {
    let count = sqlx::query_scalar(sql)
        .bind(clinic_id)
        .bind(value)
        .fetch_one(db)
        .await?;
    Ok(count)
}
