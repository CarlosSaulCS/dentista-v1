import { invokeCommand } from "@/lib/api";

export type BackupResult = {
  id: string;
  path: string;
  sizeBytes: number;
  createdAt: string;
};

export function createBackup(sessionToken: string) {
  return invokeCommand<BackupResult>("create_backup", { sessionToken });
}
