import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ExternalLink, Upload } from "lucide-react";
import { useSearchParams } from "react-router-dom";
import { toast } from "sonner";
import { PageHeader } from "@/components/data/PageHeader";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { listPatients } from "@/features/patients/services/patient-service";
import { formatDateTime } from "@/lib/api";
import { officeApi } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

export function FilesPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken) ?? "";
  const queryClient = useQueryClient();
  const [searchParams, setSearchParams] = useSearchParams();
  const [patientId, setPatientId] = useState(() => searchParams.get("patientId") ?? "");
  const [categoryName, setCategoryName] = useState("Radiografías");
  const [description, setDescription] = useState("");
  const files = useQuery({
    queryKey: ["patient-files", sessionToken],
    queryFn: () => officeApi.listPatientFiles(sessionToken),
    enabled: Boolean(sessionToken),
  });
  const patients = useQuery({
    queryKey: ["patients", sessionToken, "files"],
    queryFn: () => listPatients(sessionToken, "", 200),
    enabled: Boolean(sessionToken),
  });
  const mutation = useMutation({
    mutationFn: async (file: File) =>
      officeApi.savePatientFile(sessionToken, {
        patientId,
        categoryName: categoryName.trim(),
        originalName: file.name,
        mimeType: file.type,
        description,
        bytes: Array.from(new Uint8Array(await file.arrayBuffer())),
      }),
    onSuccess: async () => {
      toast.success("Archivo guardado localmente");
      setDescription("");
      await queryClient.invalidateQueries({ queryKey: ["patient-files"] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const handlePatientChange = (value: string) => {
    setPatientId(value);
    const nextParams = new URLSearchParams(searchParams);
    nextParams.set("patientId", value);
    setSearchParams(nextParams, { replace: true });
  };

  const handleFileSelected = (file?: File) => {
    if (!file) {
      return;
    }
    if (!patientId) {
      toast.error("Selecciona un paciente antes de subir archivos");
      return;
    }
    if (!categoryName.trim()) {
      toast.error("Escribe una categoría para clasificar el archivo");
      return;
    }
    mutation.mutate(file);
  };

  const visibleFiles = patientId
    ? (files.data ?? []).filter((file) => file.patientId === patientId)
    : (files.data ?? []);
  const uploadDisabled = !patientId || mutation.isPending;

  return (
    <div className="space-y-6">
      <PageHeader title="Archivos del paciente" description="Radiografías, fotografías, PDFs y documentos guardados en almacenamiento local." />
      <Card>
        <CardContent className="grid gap-4 p-4 md:grid-cols-[1fr_220px_1fr_160px] md:items-end">
          <div className="grid gap-2">
            <Label>Paciente</Label>
            <Select value={patientId} onValueChange={handlePatientChange}>
              <SelectTrigger>
                <SelectValue placeholder="Selecciona paciente" />
              </SelectTrigger>
              <SelectContent>
                {(patients.data ?? []).map((patient) => (
                  <SelectItem key={patient.id} value={patient.id}>
                    {patient.fullName}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="grid gap-2">
            <Label>Categoría</Label>
            <Input value={categoryName} onChange={(event) => setCategoryName(event.target.value)} />
          </div>
          <div className="grid gap-2">
            <Label>Descripción</Label>
            <Input value={description} onChange={(event) => setDescription(event.target.value)} />
          </div>
          <Button asChild disabled={uploadDisabled}>
            <label aria-disabled={uploadDisabled} className={uploadDisabled ? "pointer-events-none opacity-50" : undefined}>
              <Upload className="h-4 w-4" />
              Subir
              <Input
                className="hidden"
                type="file"
                disabled={uploadDisabled}
                onChange={(event) => {
                  handleFileSelected(event.target.files?.[0]);
                  event.currentTarget.value = "";
                }}
              />
            </label>
          </Button>
        </CardContent>
      </Card>
      <Card>
        <CardContent className="overflow-x-auto p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Paciente</TableHead>
                <TableHead>Archivo</TableHead>
                <TableHead>Categoría</TableHead>
                <TableHead>Tamaño</TableHead>
                <TableHead>Fecha</TableHead>
                <TableHead>Acción</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {visibleFiles.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={6} className="h-24 text-center text-muted-foreground">
                    No hay archivos guardados para este filtro.
                  </TableCell>
                </TableRow>
              ) : (
                visibleFiles.map((file) => (
                  <TableRow key={file.id}>
                    <TableCell>{file.patientName}</TableCell>
                    <TableCell className="font-medium">{file.originalName}</TableCell>
                    <TableCell>{file.categoryName}</TableCell>
                    <TableCell>{(file.sizeBytes / 1024).toFixed(1)} KB</TableCell>
                    <TableCell>{formatDateTime(file.createdAt)}</TableCell>
                    <TableCell>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => {
                          officeApi.openPatientFile(sessionToken, file.id).catch((error) => {
                            toast.error(error instanceof Error ? error.message : String(error));
                          });
                        }}
                      >
                        <ExternalLink className="h-4 w-4" />
                        Abrir
                      </Button>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  );
}
