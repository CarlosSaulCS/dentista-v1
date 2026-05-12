import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Archive, HardDriveDownload } from "lucide-react";
import { toast } from "sonner";
import { PageHeader } from "@/components/data/PageHeader";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { createBackup } from "@/features/backups/services/backup-service";
import { formatDateTime } from "@/lib/api";
import { useAuthStore } from "@/store/auth-store";

export function BackupsPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken);
  const queryClient = useQueryClient();
  const mutation = useMutation({
    mutationFn: () => createBackup(sessionToken ?? ""),
    onSuccess: async (backup) => {
      toast.success(`Respaldo creado: ${backup.path}`);
      await queryClient.invalidateQueries({ queryKey: ["alerts"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard-summary"] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  return (
    <div className="space-y-6">
      <PageHeader
        title="Respaldos"
        description="Crea paquetes locales con base SQLite, archivos clínicos y manifiesto de versión."
        actions={
          <Button onClick={() => mutation.mutate()} disabled={mutation.isPending}>
            <HardDriveDownload className="h-4 w-4" />
            Crear respaldo
          </Button>
        }
      />

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Archive className="h-5 w-5 text-primary" />
            Último resultado
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-2 text-sm">
          {mutation.data ? (
            <>
              <div>
                <span className="font-medium">Archivo:</span> {mutation.data.path}
              </div>
              <div>
                <span className="font-medium">Tamaño:</span> {(mutation.data.sizeBytes / 1024 / 1024).toFixed(2)} MB
              </div>
              <div>
                <span className="font-medium">Creado:</span> {formatDateTime(mutation.data.createdAt)}
              </div>
            </>
          ) : (
            <p className="text-muted-foreground">
              Los respaldos se guardan por defecto en Documentos/DentalCare Backups.
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
