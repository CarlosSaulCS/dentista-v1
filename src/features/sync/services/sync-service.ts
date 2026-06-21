import { invokeCommand } from "@/lib/api";

export interface SyncDeviceSummary {
  id: string;
  installationId: string;
  deviceId: string;
  deviceLabel?: string | null;
  portalBaseUrl?: string | null;
  status: string;
  lastRegisteredAt?: string | null;
  lastRefreshedAt?: string | null;
  lastSyncAt?: string | null;
  lastError?: string | null;
  revokedAt?: string | null;
  updatedAt: string;
}

export interface SyncStatus {
  configured: boolean;
  activeDevice?: SyncDeviceSummary | null;
  devices: SyncDeviceSummary[];
  pendingOutbox: number;
  failedOutbox: number;
  pendingInbox: number;
  pendingReceipts: number;
  pullCursor?: string | null;
  lastSyncAt?: string | null;
  lastError?: string | null;
}

export interface RegisterInstallationInput {
  portalBaseUrl: string;
  pairingCode?: string;
  deviceLabel?: string;
}

export interface RegisterInstallationResult {
  status: SyncStatus;
  installationId: string;
  deviceId: string;
  paired: boolean;
}

export interface SyncRunResult {
  pushedEvents: number;
  appliedCommands: number;
  failedCommands: number;
  ackedReceipts: number;
  status: SyncStatus;
}

export function getSyncStatus(sessionToken: string) {
  return invokeCommand<SyncStatus>("get_sync_status", { sessionToken });
}

export function registerInstallation(sessionToken: string, input: RegisterInstallationInput) {
  return invokeCommand<RegisterInstallationResult>("register_installation", {
    sessionToken,
    input,
  });
}

export function refreshSyncToken(sessionToken: string) {
  return invokeCommand<SyncStatus>("refresh_sync_token", { sessionToken });
}

export function syncNow(sessionToken: string) {
  return invokeCommand<SyncRunResult>("sync_now", { sessionToken });
}

export function revokeLocalDevice(sessionToken: string, deviceId: string, reason?: string) {
  return invokeCommand<SyncStatus>("revoke_local_device", {
    sessionToken,
    input: { deviceId, reason },
  });
}
