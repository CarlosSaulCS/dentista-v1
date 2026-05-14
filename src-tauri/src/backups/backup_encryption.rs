//! Interfaz preparada para cifrado de respaldos.
//!
//! La fase local actual escribe respaldos verificables sin cifrado (`encrypted: false`).
//! Este módulo deja un contrato explícito para agregar cifrado autenticado antes de
//! subir respaldos a almacenamiento remoto o transportar ZIPs fuera del consultorio.

use crate::errors::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct BackupEncryptionRequest {
    pub key_id: String,
    pub plaintext: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct BackupEncryptionResult {
    pub key_id: String,
    pub ciphertext: Vec<u8>,
    pub algorithm: String,
}

pub fn encrypt_backup_payload(
    _request: BackupEncryptionRequest,
) -> AppResult<BackupEncryptionResult> {
    Err(AppError::Conflict(
        "El cifrado de respaldos aún no está habilitado en esta fase local".to_string(),
    ))
}
