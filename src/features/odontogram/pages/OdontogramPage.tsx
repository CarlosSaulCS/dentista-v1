import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useSearchParams } from "react-router-dom";
import { toast } from "sonner";
import { EmptyState } from "@/components/data/EmptyState";
import { PageHeader } from "@/components/data/PageHeader";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { OdontogramGrid } from "@/features/odontogram/components/OdontogramGrid";
import { getOdontogram, upsertOdontogramEntry } from "@/features/odontogram/services/odontogram-service";
import { listPatients } from "@/features/patients/services/patient-service";
import { useAuthStore } from "@/store/auth-store";

const dentalStates = [
  "sano",
  "caries",
  "restauracion",
  "obturacion",
  "corona",
  "endodoncia",
  "ausente",
  "extraccion_indicada",
  "implante",
  "protesis",
  "sellador",
  "movilidad",
  "fractura",
  "en_observacion",
  "tratamiento_pendiente",
  "tratamiento_realizado",
];

export function OdontogramPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const [patientId, setPatientId] = useState(() => searchParams.get("patientId") ?? "");
  const [selectedTooth, setSelectedTooth] = useState("11");
  const [state, setState] = useState("sano");
  const [finding, setFinding] = useState("");
  const sessionToken = useAuthStore((auth) => auth.sessionToken);
  const queryClient = useQueryClient();
  const dentitionType = "permanent";

  const patients = useQuery({
    queryKey: ["patients", sessionToken, "odontogram-picker"],
    queryFn: () => listPatients(sessionToken ?? "", "", 200),
    enabled: Boolean(sessionToken),
  });

  const odontogram = useQuery({
    queryKey: ["odontogram", sessionToken, patientId, dentitionType],
    queryFn: () => getOdontogram(sessionToken ?? "", patientId, dentitionType),
    enabled: Boolean(sessionToken && patientId),
  });

  const selectedEntry = useMemo(
    () => odontogram.data?.entries.find((entry) => entry.toothNumber === selectedTooth),
    [odontogram.data?.entries, selectedTooth],
  );

  const mutation = useMutation({
    mutationFn: () =>
      upsertOdontogramEntry(sessionToken ?? "", {
        patientId,
        dentitionType,
        toothNumber: selectedTooth,
        surface: "all",
        state,
        finding,
      }),
    onSuccess: async () => {
      toast.success("Odontograma actualizado");
      await queryClient.invalidateQueries({ queryKey: ["odontogram"] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const handlePatientChange = (value: string) => {
    setPatientId(value);
    setSelectedTooth("11");
    setState("sano");
    setFinding("");
    const nextParams = new URLSearchParams(searchParams);
    nextParams.set("patientId", value);
    setSearchParams(nextParams, { replace: true });
  };

  return (
    <div className="space-y-6">
      <PageHeader title="Odontograma" description="Registro visual por pieza dental con historial de cambios en SQLite." />

      <Card>
        <CardContent className="grid gap-2 p-4">
          <Label>Paciente</Label>
          <Select value={patientId} onValueChange={handlePatientChange}>
            <SelectTrigger className="max-w-xl">
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
        </CardContent>
      </Card>

      {!patientId ? (
        <EmptyState title="Selecciona un paciente" description="El odontograma se guarda como parte del expediente clínico." />
      ) : (
        <div className="grid gap-4 xl:grid-cols-[1fr_340px]">
          <Card>
            <CardHeader>
              <CardTitle>Dentición permanente</CardTitle>
            </CardHeader>
            <CardContent>
              <OdontogramGrid
                entries={odontogram.data?.entries ?? []}
                selectedTooth={selectedTooth}
                onSelect={(tooth) => {
                  setSelectedTooth(tooth);
                  const entry = odontogram.data?.entries.find((item) => item.toothNumber === tooth);
                  setState(entry?.state ?? "sano");
                  setFinding(entry?.finding ?? "");
                }}
              />
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Pieza {selectedTooth}</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-4">
              <div className="grid gap-2">
                <Label>Estado</Label>
                <Select value={state} onValueChange={setState}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {dentalStates.map((item) => (
                      <SelectItem key={item} value={item}>
                        {item.replaceAll("_", " ")}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="grid gap-2">
                <Label>Hallazgo</Label>
                <Textarea value={finding} onChange={(event) => setFinding(event.target.value)} />
              </div>
              <div className="rounded-md bg-muted p-3 text-sm text-muted-foreground">
                Registro actual: {selectedEntry?.state?.replaceAll("_", " ") ?? "sin hallazgo"}
              </div>
              <Button onClick={() => mutation.mutate()} disabled={mutation.isPending}>
                Guardar pieza
              </Button>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
}
