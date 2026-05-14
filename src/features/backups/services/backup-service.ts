import { invokeCommand } from "@/lib/api";

export type BackupResult = {
  id: string;
  path: string;
  sizeBytes: number;
  createdAt: string;
  checksumSha256?: string | null;
};

export type BackupSummary = BackupResult & {
  status: string;
  backupType: string;
  checksumSha256?: string | null;
  verificationStatus?: string | null;
  verifiedAt?: string | null;
  appVersion?: string | null;
  migrationVersion?: number | null;
  fileCount: number;
};

export type BackupSettings = {
  automaticEnabled: boolean;
  frequency: string;
  includeFiles: boolean;
  encryptBackups: boolean;
  retentionLimit: number;
  updatedAt?: string | null;
};

export type BackupManifest = {
  backupId: string;
  clinicId?: string | null;
  createdAt: string;
  appVersion: string;
  databaseVersion: string;
  migrationVersion: number;
  includes: string[];
  tableCounts: Record<string, number>;
  fileCount: number;
  checksum: Record<string, unknown>;
  compression: string;
  encrypted: boolean;
  createdByUserId?: string | null;
};

export type BackupVerificationResult = {
  valid: boolean;
  path: string;
  backupId?: string | null;
  archiveChecksumSha256?: string | null;
  manifest?: BackupManifest | null;
  checkedFiles: number;
  errors: string[];
};

export type RestorePreview = {
  valid: boolean;
  sourcePath: string;
  manifest?: BackupManifest | null;
  summary: Record<string, unknown>;
  errors: string[];
};

export type RestorePrepareResult = {
  restoreJobId: string;
  stagedPath: string;
  safetyBackupPath: string;
  message: string;
};

export function createBackup(sessionToken: string) {
  return invokeCommand<BackupResult>("create_backup", { sessionToken });
}

export function listBackups(sessionToken: string) {
  return invokeCommand<BackupSummary[]>("list_backups", { sessionToken });
}

export function verifyBackup(sessionToken: string, path: string) {
  return invokeCommand<BackupVerificationResult>("verify_backup", { sessionToken, path });
}

export function previewRestore(sessionToken: string, path: string) {
  return invokeCommand<RestorePreview>("preview_restore", { sessionToken, path });
}

export function prepareRestore(sessionToken: string, path: string) {
  return invokeCommand<RestorePrepareResult>("prepare_restore", { sessionToken, path });
}

export function getBackupSettings(sessionToken: string) {
  return invokeCommand<BackupSettings>("get_backup_settings", { sessionToken });
}

export function updateBackupSettings(sessionToken: string, input: BackupSettings) {
  return invokeCommand<BackupSettings>("update_backup_settings", { sessionToken, input });
}
