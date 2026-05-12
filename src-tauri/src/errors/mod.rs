use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Base de datos: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Migraciones: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("Archivo: {0}")]
    Io(#[from] std::io::Error),
    #[error("Compresión: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Configuración de la aplicación: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("No autorizado")]
    Unauthorized,
    #[error("Permiso insuficiente: {0}")]
    PermissionDenied(String),
    #[error("Validación: {0}")]
    Validation(String),
    #[error("Operación no permitida: {0}")]
    Conflict(String),
}

pub type AppResult<T> = Result<T, AppError>;

pub fn command_error(error: AppError) -> String {
    error.to_string()
}
