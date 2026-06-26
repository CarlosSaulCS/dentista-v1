import { useState } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Archive, CheckCircle2, HardDriveDownload, RotateCcw, ShieldCheck } from "lucide-react";
import { toast } from "sonner";
import { PageHeader } from "@/components/data/PageHeader";
import { StatusBadge } from "@/components/data/StatusBadge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import {
  createBackup,
  getBackupSettings,
  listBackups,
  prepareRestore,
  previewRestore,
  updateBackupSettings,
  verifyBackup,
  type BackupSettings,
  type RestorePreview,
} from "@/features/backups/services/backup-service";
import { formatDateTime } from "@/lib/api";
import { useAuthStore } from "@/store/auth-store";

export function BackupsPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken) ?? "";
  const queryClient = useQueryClient();
  const [selectedRestorePath, setSelectedRestorePath] = useState("");
  const [restorePreview, setRestorePreview] = useState<RestorePreview | null>(null);

  const backups = useQuery({
    queryKey: ["backups", sessionToken],
    queryFn: () => listBackups(sessionToken),
    enabled: Boolean(sessionToken),
  });
  const settings = useQuery({
    queryKey: ["backup-settings", sessionToken],
    queryFn: () => getBackupSettings(sessionToken),
    enabled: Boolean(sessionToken),
  });

  const createMutation = useMutation({
    mutationFn: () => createBackup(sessionToken),
    onSuccess: async (backup) => {
      toast.success(`Respaldo creado: ${backup.path}`);
      await invalidateBackupData(queryClient);
    },
    onError: showError,
  });
  const verifyMutation = useMutation({
    mutationFn: (path: string) => verifyBackup(sessionToken, path),
    onSuccess: async (result) => {
      toast[result.valid ? "success" : "error"](
        result.valid ? "Respaldo verificado correctamente" : result.errors.join(", "),
      );
      await invalidateBackupData(queryClient);
    },
    onError: showError,
  });
  const previewMutation = useMutation({
    mutationFn: (path: string) => previewRestore(sessionToken, path),
    onSuccess: (preview) => {
      setRestorePreview(preview);
      toast[preview.valid ? "success" : "error"](preview.valid ? "Respaldo listo para restauración staged" : preview.errors.join(", "));
    },
    onError: showError,
  });
  const prepareMutation = useMutation({
    mutationFn: (path: string) => prepareRestore(sessionToken, path),
    onSuccess: async (result) => {
      toast.success(result.message);
      await invalidateBackupData(queryClient);
    },
    onError: showError,
  });
  const settingsMutation = useMutation({
    mutationFn: (input: BackupSettings) => updateBackupSettings(sessionToken, input),
    onSuccess: async () => {
      toast.success("Configuración de respaldos guardada");
      await queryClient.invalidateQueries({ queryKey: ["backup-settings"] });
    },
    onError: showError,
  });

  const pickRestoreFile = async () => {
    const selected = await openDialog({
      title: "Seleccionar respaldo Dentista v1",
      multiple: false,
      filters: [{ name: "Dentista v1 backup", extensions: ["zip"] }],
    });
    if (typeof selected === "string") {
      setSelectedRestorePath(selected);
      setRestorePreview(null);
      previewMutation.mutate(selected);
    }
  };

  const currentSettings = settings.data;

  return (
    <div className="space-y-6">
      <PageHeader
        title="Respaldos"
        description="ZIP local verificable con base SQLite, archivos clínicos, manifiesto y checksums SHA-256."
        actions={
          <Button onClick={() => createMutation.mutate()} disabled={createMutation.isPending}>
            <HardDriveDownload className="h-4 w-4" />
            Crear respaldo
          </Button>
        }
      />

      <div className="grid gap-4 xl:grid-cols-[1fr_360px]">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Archive className="h-5 w-5 text-primary" />
              Historial local
            </CardTitle>
          </CardHeader>
          <CardContent className="overflow-x-auto p-0">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Fecha</TableHead>
                  <TableHead>Tipo</TableHead>
                  <TableHead>Tamaño</TableHead>
                  <TableHead>Verificación</TableHead>
                  <TableHead>Archivos</TableHead>
                  <TableHead>Ruta</TableHead>
                  <TableHead></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {(backups.data ?? []).length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={7} className="h-24 text-center text-muted-foreground">
                      Aún no hay respaldos registrados.
                    </TableCell>
                  </TableRow>
                ) : (
                  (backups.data ?? []).map((backup) => (
                    <TableRow key={backup.id}>
                      <TableCell>{formatDateTime(backup.createdAt)}</TableCell>
                      <TableCell><StatusBadge status={backup.backupType} /></TableCell>
                      <TableCell>{(backup.sizeBytes / 1024 / 1024).toFixed(2)} MB</TableCell>
                      <TableCell>{backup.verificationStatus ?? "no_verificado"}</TableCell>
                      <TableCell>{backup.fileCount}</TableCell>
                      <TableCell className="max-w-[360px] truncate text-xs text-muted-foreground">{backup.path}</TableCell>
                      <TableCell>
                        <Button variant="outline" size="sm" onClick={() => verifyMutation.mutate(backup.path)} disabled={verifyMutation.isPending}>
                          <ShieldCheck className="h-4 w-4" />
                          Verificar
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))
                )}
              </TableBody>
            </Table>
          </CardContent>
        </Card>

        <div className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Automático</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-2">
                <Label>Frecuencia</Label>
                <Select
                  value={currentSettings?.frequency ?? "daily"}
                  onValueChange={(frequency) => currentSettings && settingsMutation.mutate({ ...currentSettings, frequency })}
                >
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="daily">Diario al iniciar</SelectItem>
                    <SelectItem value="weekly">Semanal</SelectItem>
                    <SelectItem value="manual">Sólo manual</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="grid gap-2">
                <Label>Retención</Label>
                <Input
                  type="number"
                  min={1}
                  max={365}
                  value={currentSettings?.retentionLimit ?? 30}
                  onChange={(event) => currentSettings && settingsMutation.mutate({ ...currentSettings, retentionLimit: Number(event.target.value) || 30 })}
                />
              </div>
              <Button
                variant="outline"
                onClick={() => currentSettings && settingsMutation.mutate({ ...currentSettings, automaticEnabled: !currentSettings.automaticEnabled })}
                disabled={!currentSettings || settingsMutation.isPending}
              >
                {currentSettings?.automaticEnabled ? "Desactivar automático" : "Activar automático"}
              </Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <RotateCcw className="h-5 w-5 text-primary" />
                Restauración staged
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3 text-sm">
              <Button variant="outline" onClick={() => void pickRestoreFile()} disabled={previewMutation.isPending}>
                Seleccionar ZIP
              </Button>
              {selectedRestorePath ? <p className="break-all text-xs text-muted-foreground">{selectedRestorePath}</p> : null}
              {restorePreview ? (
                <div className="rounded-md border p-3">
                  <div className="flex items-center gap-2 font-medium">
                    <CheckCircle2 className={restorePreview.valid ? "h-4 w-4 text-emerald-600" : "h-4 w-4 text-destructive"} />
                    {restorePreview.valid ? "Verificación válida" : "Verificación fallida"}
                  </div>
                  {restorePreview.manifest ? (
                    <p className="mt-2 text-muted-foreground">
                      {restorePreview.manifest.appVersion} · migración {restorePreview.manifest.migrationVersion} · {restorePreview.manifest.fileCount} archivo(s)
                    </p>
                  ) : null}
                  {restorePreview.errors.length > 0 ? <p className="mt-2 text-destructive">{restorePreview.errors.join(", ")}</p> : null}
                </div>
              ) : null}
              <Button
                onClick={() => selectedRestorePath && prepareMutation.mutate(selectedRestorePath)}
                disabled={!restorePreview?.valid || prepareMutation.isPending}
              >
                Preparar restauración
              </Button>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}

async function invalidateBackupData(queryClient: ReturnType<typeof useQueryClient>) {
  await queryClient.invalidateQueries({ queryKey: ["backups"] });
  await queryClient.invalidateQueries({ queryKey: ["alerts"] });
  await queryClient.invalidateQueries({ queryKey: ["dashboard-summary"] });
}

function showError(error: unknown) {
  toast.error(error instanceof Error ? error.message : String(error));
}
