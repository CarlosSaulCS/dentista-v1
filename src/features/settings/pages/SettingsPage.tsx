import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { RefreshCw, RotateCcw, Unlink } from "lucide-react";
import { toast } from "sonner";
import { PageHeader } from "@/components/data/PageHeader";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { getBootstrapStatus } from "@/features/auth/services/auth-service";
import {
  getSyncStatus,
  refreshSyncToken,
  registerInstallation,
  revokeLocalDevice,
  syncNow,
  type SyncDeviceSummary,
} from "@/features/sync/services/sync-service";
import { formatDateTime } from "@/lib/api";
import { officeApi } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

export function SettingsPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken) ?? "";
  const queryClient = useQueryClient();
  const bootstrap = useQuery({ queryKey: ["bootstrap-status"], queryFn: getBootstrapStatus });
  const templates = useQuery({
    queryKey: ["message-templates", sessionToken],
    queryFn: () => officeApi.listMessageTemplates(sessionToken),
    enabled: Boolean(sessionToken),
  });
  const syncStatus = useQuery({
    queryKey: ["sync-status", sessionToken],
    queryFn: () => getSyncStatus(sessionToken),
    enabled: Boolean(sessionToken),
  });
  const clinic = bootstrap.data?.clinic;
  const license = bootstrap.data?.license;
  const [draft, setDraft] = useState({ name: "", phone: "", whatsapp: "", email: "", address: "", taxData: "" });
  const [syncDraft, setSyncDraft] = useState({ portalBaseUrl: "", pairingCode: "", deviceLabel: "" });

  const settingsMutation = useMutation({
    mutationFn: () =>
      officeApi.updateClinicSettings(sessionToken, {
        ...draft,
        name: draft.name || clinic?.name || "",
        phone: draft.phone || clinic?.phone || "",
        whatsapp: draft.whatsapp || clinic?.whatsapp || "",
        email: draft.email || clinic?.email || "",
        address: draft.address || clinic?.address || "",
      }),
    onSuccess: async () => {
      toast.success("Configuración guardada");
      await queryClient.invalidateQueries({ queryKey: ["bootstrap-status"] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const registerMutation = useMutation({
    mutationFn: () => registerInstallation(sessionToken, syncDraft),
    onSuccess: async (result) => {
      toast.success(result.paired ? "Dispositivo vinculado" : "Dispositivo preparado para pairing");
      setSyncDraft((current) => ({ ...current, pairingCode: "" }));
      await queryClient.invalidateQueries({ queryKey: ["sync-status", sessionToken] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const runSyncMutation = useMutation({
    mutationFn: () => syncNow(sessionToken),
    onSuccess: async (result) => {
      toast.success(`Sync: ${result.pushedEvents} eventos, ${result.appliedCommands} comandos`);
      await queryClient.invalidateQueries({ queryKey: ["sync-status", sessionToken] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const refreshMutation = useMutation({
    mutationFn: () => refreshSyncToken(sessionToken),
    onSuccess: async () => {
      toast.success("Token actualizado");
      await queryClient.invalidateQueries({ queryKey: ["sync-status", sessionToken] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const revokeMutation = useMutation({
    mutationFn: (deviceId: string) => revokeLocalDevice(sessionToken, deviceId, "Revocado manualmente"),
    onSuccess: async () => {
      toast.success("Dispositivo revocado");
      await queryClient.invalidateQueries({ queryKey: ["sync-status", sessionToken] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const currentSync = syncStatus.data;
  const activeDevice = currentSync?.activeDevice;

  return (
    <div className="space-y-6">
      <PageHeader title="Configuración" description="Datos locales del consultorio, licencia, conectividad y plantillas." />

      {license ? (
        <Card>
          <CardHeader>
            <CardTitle>Licencia local</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            <p className="font-medium">
              {license.canWrite ? "Operación completa" : "Modo sólo lectura"} · {license.status.replaceAll("_", " ")}
            </p>
            <p className="text-muted-foreground">{license.message}</p>
            {!license.isLicensed && license.trialEndsAt ? (
              <p className="text-muted-foreground">
                Vigencia de prueba: {formatLicenseDate(license.trialEndsAt)}. Días restantes: {license.daysRemaining}.
              </p>
            ) : null}
            {license.isLicensed && license.activatedAt ? (
              <p className="text-muted-foreground">Activado el {formatLicenseDate(license.activatedAt)}.</p>
            ) : null}
            {license.nextCheckAt ? (
              <p className="text-muted-foreground">Próxima validación preparada: {formatLicenseDate(license.nextCheckAt)}.</p>
            ) : null}
            {license.installationId ? <p className="break-all text-xs text-muted-foreground">Instalación: {license.installationId}</p> : null}
          </CardContent>
        </Card>
      ) : null}

      <Card>
        <CardHeader>
          <CardTitle>Conectividad</CardTitle>
        </CardHeader>
        <CardContent className="grid gap-5">
          <div className="grid gap-3 md:grid-cols-4">
            <SyncMetric label="Estado" value={activeDevice ? statusLabel(activeDevice.status) : "Sin vínculo"} />
            <SyncMetric label="Última sincronización" value={currentSync?.lastSyncAt ? formatDateTime(currentSync.lastSyncAt) : "Pendiente"} />
            <SyncMetric label="Outbox" value={`${currentSync?.pendingOutbox ?? 0} pendientes`} />
            <SyncMetric label="Comandos" value={`${currentSync?.pendingInbox ?? 0} inbox · ${currentSync?.pendingReceipts ?? 0} ack`} />
          </div>

          {currentSync?.lastError ? (
            <div className="rounded-md border border-destructive/30 bg-destructive/5 px-3 py-2 text-sm text-destructive">
              {currentSync.lastError}
            </div>
          ) : null}

          <div className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
            <div className="grid gap-3 md:grid-cols-3">
              <Field
                label="URL del portal"
                value={syncDraft.portalBaseUrl}
                placeholder={activeDevice?.portalBaseUrl ?? "https://..."}
                onChange={(portalBaseUrl) => setSyncDraft({ ...syncDraft, portalBaseUrl })}
              />
              <Field
                label="Código de pairing"
                value={syncDraft.pairingCode}
                onChange={(pairingCode) => setSyncDraft({ ...syncDraft, pairingCode })}
              />
              <Field
                label="Nombre del dispositivo"
                value={syncDraft.deviceLabel}
                placeholder={activeDevice?.deviceLabel ?? "Recepción"}
                onChange={(deviceLabel) => setSyncDraft({ ...syncDraft, deviceLabel })}
              />
            </div>
            <div className="flex flex-wrap gap-2">
              <Button type="button" onClick={() => registerMutation.mutate()} disabled={registerMutation.isPending}>
                <RefreshCw className="h-4 w-4" />
                Vincular
              </Button>
              <Button type="button" variant="outline" onClick={() => runSyncMutation.mutate()} disabled={!activeDevice || runSyncMutation.isPending}>
                <RotateCcw className="h-4 w-4" />
                Sincronizar
              </Button>
              <Button type="button" variant="outline" onClick={() => refreshMutation.mutate()} disabled={!activeDevice || refreshMutation.isPending}>
                <RefreshCw className="h-4 w-4" />
                Token
              </Button>
            </div>
          </div>

          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Dispositivo</TableHead>
                <TableHead>Estado</TableHead>
                <TableHead>Portal</TableHead>
                <TableHead>Último sync</TableHead>
                <TableHead className="w-[80px]" />
              </TableRow>
            </TableHeader>
            <TableBody>
              {(currentSync?.devices ?? []).map((device) => (
                <TableRow key={device.id}>
                  <TableCell className="min-w-[220px]">
                    <p className="font-medium">{device.deviceLabel || "Dentista v1 Professional"}</p>
                    <p className="break-all text-xs text-muted-foreground">{device.deviceId}</p>
                  </TableCell>
                  <TableCell>
                    <Badge variant={device.status === "active" ? "default" : "outline"}>{statusLabel(device.status)}</Badge>
                  </TableCell>
                  <TableCell className="max-w-[260px] truncate">{device.portalBaseUrl ?? "Sin portal"}</TableCell>
                  <TableCell>{device.lastSyncAt ? formatDateTime(device.lastSyncAt) : "Pendiente"}</TableCell>
                  <TableCell>
                    <Button
                      type="button"
                      size="icon"
                      variant="ghost"
                      disabled={device.status === "revoked" || revokeMutation.isPending}
                      onClick={() => revokeMutation.mutate(device.deviceId)}
                      aria-label="Revocar dispositivo"
                    >
                      <Unlink className="h-4 w-4" />
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
              {(currentSync?.devices ?? []).length === 0 ? (
                <TableRow>
                  <TableCell colSpan={5} className="text-sm text-muted-foreground">
                    Sin dispositivos vinculados.
                  </TableCell>
                </TableRow>
              ) : null}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Consultorio</CardTitle>
        </CardHeader>
        <CardContent className="grid gap-4 md:grid-cols-2">
          <Field label="Nombre" value={draft.name || clinic?.name || ""} onChange={(name) => setDraft({ ...draft, name })} />
          <Field label="Teléfono" value={draft.phone || ""} onChange={(phone) => setDraft({ ...draft, phone })} />
          <Field label="WhatsApp" value={draft.whatsapp || ""} onChange={(whatsapp) => setDraft({ ...draft, whatsapp })} />
          <Field label="Correo" value={draft.email || ""} onChange={(email) => setDraft({ ...draft, email })} />
          <Field label="Dirección" value={draft.address || ""} onChange={(address) => setDraft({ ...draft, address })} />
          <Field label="Datos fiscales" value={draft.taxData || ""} onChange={(taxData) => setDraft({ ...draft, taxData })} />
          <Button className="md:col-span-2" onClick={() => settingsMutation.mutate()} disabled={settingsMutation.isPending}>
            Guardar configuración
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Plantillas de WhatsApp listas para copiar</CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Nombre</TableHead>
                <TableHead>Texto</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {(templates.data ?? []).map((template) => (
                <TableRow key={template.id}>
                  <TableCell className="font-medium">{template.name}</TableCell>
                  <TableCell>{template.body}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  );
}

function SyncMetric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border bg-muted/20 p-3">
      <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">{label}</p>
      <p className="mt-1 break-words text-sm font-medium">{value}</p>
    </div>
  );
}

function Field({
  label,
  value,
  onChange,
  placeholder,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}) {
  return (
    <div className="grid gap-2">
      <Label>{label}</Label>
      <Input value={value} placeholder={placeholder} onChange={(event) => onChange(event.target.value)} />
    </div>
  );
}

function formatLicenseDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value.replace("T", " ").slice(0, 16);
  return new Intl.DateTimeFormat("es-MX", { dateStyle: "medium", timeStyle: "short" }).format(date);
}

function statusLabel(status: SyncDeviceSummary["status"]) {
  if (status === "active") return "Activo";
  if (status === "pending_pairing") return "Pairing";
  if (status === "revoked") return "Revocado";
  if (status === "error") return "Error";
  return status;
}
