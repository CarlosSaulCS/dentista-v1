import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Activity, CalendarPlus, ClipboardPlus, Plus, Search, WalletCards } from "lucide-react";
import { Link } from "react-router-dom";
import { toast } from "sonner";
import { EmptyState } from "@/components/data/EmptyState";
import { PageHeader } from "@/components/data/PageHeader";
import { StatusBadge } from "@/components/data/StatusBadge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from "@/components/ui/sheet";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { PatientForm } from "@/features/patients/components/PatientForm";
import { createPatient, listPatients } from "@/features/patients/services/patient-service";
import type { PatientFormValues } from "@/features/patients/schemas/patient-schemas";
import { useAuthStore } from "@/store/auth-store";

export function PatientsPage() {
  const [search, setSearch] = useState("");
  const [open, setOpen] = useState(false);
  const sessionToken = useAuthStore((state) => state.sessionToken);
  const queryClient = useQueryClient();
  const queryKey = useMemo(() => ["patients", sessionToken, search], [search, sessionToken]);

  const { data = [], isLoading } = useQuery({
    queryKey,
    queryFn: () => listPatients(sessionToken ?? "", search),
    enabled: Boolean(sessionToken),
  });

  const mutation = useMutation({
    mutationFn: (values: PatientFormValues) => createPatient(sessionToken ?? "", values),
    onSuccess: async () => {
      toast.success("Paciente registrado");
      setOpen(false);
      await queryClient.invalidateQueries({ queryKey: ["patients"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard-summary"] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  return (
    <div className="space-y-6">
      <PageHeader
        title="Pacientes"
        description="Registro clínico-administrativo único por paciente."
        actions={
          <Sheet open={open} onOpenChange={setOpen}>
            <SheetTrigger asChild>
              <Button>
                <Plus className="h-4 w-4" />
                Nuevo paciente
              </Button>
            </SheetTrigger>
            <SheetContent className="w-[720px] overflow-y-auto sm:max-w-[720px]">
              <SheetHeader>
                <SheetTitle>Registrar paciente</SheetTitle>
              </SheetHeader>
              <div className="mt-6">
                <PatientForm
                  onSubmit={async (values) => {
                    await mutation.mutateAsync(values);
                  }}
                  submitting={mutation.isPending}
                />
              </div>
            </SheetContent>
          </Sheet>
        }
      />

      <div className="flex max-w-md items-center gap-2">
        <Search className="h-4 w-4 text-muted-foreground" />
        <Input placeholder="Buscar por nombre, teléfono, WhatsApp o correo" value={search} onChange={(event) => setSearch(event.target.value)} />
      </div>

      {data.length === 0 && !isLoading ? (
        <EmptyState title="Sin pacientes registrados" description="Crea el primer paciente para iniciar agenda, expediente y odontograma." />
      ) : (
        <div className="overflow-x-auto rounded-lg border bg-card">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Paciente</TableHead>
                <TableHead>Contacto</TableHead>
                <TableHead>Riesgos clínicos</TableHead>
                <TableHead>Estado</TableHead>
                <TableHead className="w-[390px]">Acciones rápidas</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.map((patient) => {
                const patientParam = encodeURIComponent(patient.id);
                return (
                  <TableRow key={patient.id}>
                    <TableCell>
                      <div className="font-medium">{patient.fullName}</div>
                      <div className="text-xs text-muted-foreground">{patient.email || "Sin correo"}</div>
                    </TableCell>
                    <TableCell>
                      <div>{patient.phone || patient.whatsapp || "Sin teléfono"}</div>
                      <div className="text-xs text-muted-foreground">WhatsApp: {patient.whatsapp || "No registrado"}</div>
                    </TableCell>
                    <TableCell className="max-w-md">
                      <div className="truncate text-sm">{patient.allergies || "Sin alergias registradas"}</div>
                      <div className="truncate text-xs text-muted-foreground">{patient.systemicDiseases || "Sin enfermedades registradas"}</div>
                    </TableCell>
                    <TableCell>
                      <StatusBadge status={patient.status} />
                    </TableCell>
                    <TableCell>
                      <div className="flex flex-wrap gap-2">
                        <Button asChild variant="outline" size="sm">
                          <Link to={`/appointments?patientId=${patientParam}&new=1`}>
                            <CalendarPlus className="h-4 w-4" />
                            Agendar
                          </Link>
                        </Button>
                        <Button asChild variant="outline" size="sm">
                          <Link to={`/clinical-records?patientId=${patientParam}`}>
                            <ClipboardPlus className="h-4 w-4" />
                            Expediente
                          </Link>
                        </Button>
                        <Button asChild variant="outline" size="sm">
                          <Link to={`/odontogram?patientId=${patientParam}`}>
                            <Activity className="h-4 w-4" />
                            Odontograma
                          </Link>
                        </Button>
                        <Button asChild variant="outline" size="sm">
                          <Link to={`/payments?patientId=${patientParam}&new=1`}>
                            <WalletCards className="h-4 w-4" />
                            Pago
                          </Link>
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  );
}
