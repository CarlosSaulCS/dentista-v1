pub mod commands;
pub mod database;
pub mod errors;
pub mod files;
pub mod models;
pub mod reports;
pub mod repositories;
pub mod security;
pub mod services;
pub mod utils;

use commands::{
    appointments::{create_appointment, list_appointments, update_appointment_status},
    auth::{get_bootstrap_status, list_users, login, logout, setup_clinic_and_admin},
    backups::create_backup,
    clinical::{
        create_clinical_evolution, create_clinical_record, list_clinical_evolutions,
        list_clinical_records,
    },
    dashboard::get_dashboard_summary,
    odontogram::{get_odontogram, upsert_odontogram_entry},
    office::{
        close_cash_register, create_alert, create_consent_template, create_estimate,
        create_inventory_item, create_inventory_movement, create_periodontal_record,
        create_supplier, create_treatment, create_treatment_plan, create_user,
        get_current_cash_register, get_reports_summary, list_alerts, list_consent_templates,
        list_estimate_items, list_estimates, list_inventory_items, list_message_templates,
        list_patient_files, list_payments, list_periodontal_records, list_roles, list_suppliers,
        list_treatment_plan_items, list_treatment_plans, list_treatments, open_cash_register,
        register_payment, resolve_alert, save_patient_file, update_clinic_settings,
        update_estimate_status, update_treatment,
    },
    patients::{create_patient, get_patient, list_patients},
    reports::save_report_file,
};
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = tauri::async_runtime::block_on(database::init(app.handle()))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_bootstrap_status,
            setup_clinic_and_admin,
            login,
            logout,
            list_users,
            create_patient,
            list_patients,
            get_patient,
            create_appointment,
            list_appointments,
            update_appointment_status,
            create_clinical_record,
            create_clinical_evolution,
            list_clinical_records,
            list_clinical_evolutions,
            get_odontogram,
            upsert_odontogram_entry,
            get_dashboard_summary,
            create_backup,
            list_treatments,
            create_treatment,
            update_treatment,
            list_treatment_plans,
            create_treatment_plan,
            list_treatment_plan_items,
            list_estimates,
            create_estimate,
            update_estimate_status,
            list_estimate_items,
            list_payments,
            register_payment,
            get_current_cash_register,
            open_cash_register,
            close_cash_register,
            list_suppliers,
            create_supplier,
            list_inventory_items,
            create_inventory_item,
            create_inventory_movement,
            list_alerts,
            create_alert,
            resolve_alert,
            save_patient_file,
            list_patient_files,
            list_consent_templates,
            create_consent_template,
            get_reports_summary,
            save_report_file,
            update_clinic_settings,
            list_message_templates,
            list_roles,
            create_user,
            list_periodontal_records,
            create_periodontal_record
        ])
        .run(tauri::generate_context!())
        .expect("error while running DentalCare Manager");
}
