import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Activity, CalendarPlus, ClipboardPlus, Edit, Mail, MessageCircle, Plus, Search, Trash2, WalletCards } from "lucide-react";
import { Link } from "react-router-dom";
import { toast } from "sonner";
import { EmptyState } from "@/components/data/EmptyState";
import { PageHeader } from "@/components/data/PageHeader";
import { StatusBadge } from "@/components/data/StatusBadge";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from "@/components/ui/sheet";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { getNextPatientAppointment } from "@/features/appointments/services/appointment-service";
import type { AppointmentSummary } from "@/features/appointments/types/appointment-types";
import { PatientForm } from "@/features/patients/components/PatientForm";
import { createPatient, listPatients, softDeletePatient, updatePatient } from "@/features/patients/services/patient-service";
import type { PatientFormValues } from "@/features/patients/schemas/patient-schemas";
import type { PatientSummary } from "@/features/patients/types/patient-types";
import { formatDateTime } from "@/lib/api";
import { officeApi } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

export function PatientsPage() {
  const [search, setSearch] = useState("");
  const [open, setOpen] = useState(false);
  const [editingPatient, setEditingPatient] = useState<PatientSummary | null>(null);
  const sessionToken = useAuthStore((state) => state.sessionToken);
  const queryClient = useQueryClient();
  const queryKey = useMemo(() => ["patients", sessionToken, search], [search, sessionToken]);
  const editingValues = useMemo(() => (editingPatient ? patientToFormValues(editingPatient) : undefined), [editingPatient]);

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
      await invalidatePatientData(queryClient);
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const updateMutation = useMutation({
    mutationFn: (values: PatientFormValues) =>
      updatePatient(sessionToken ?? "", { ...values, id: editingPatient?.id ?? "", status: editingPatient?.status ?? "active" }),
    onSuccess: async () => {
      toast.success("Paciente actualizado");
      setEditingPatient(null);
      await invalidatePatientData(queryClient);
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const deleteMutation = useMutation({
    mutationFn: (patientId: string) => softDeletePatient(sessionToken ?? "", patientId),
    onSuccess: async () => {
      toast.success("Paciente dado de baja");
      await invalidatePatientData(queryClient);
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });
  const sendNextAppointmentMessage = async (patient: PatientSummary, channel: "whatsapp" | "email") => {
    try {
      const appointment = await getNextPatientAppointment(sessionToken ?? "", patient.id);
      if (!appointment) {
        toast.error("Este paciente no tiene una próxima cita activa. Agenda una cita primero.");
        return;
      }
      const url = channel === "whatsapp" ? buildWhatsAppUrl(appointment) : buildEmailUrl(appointment);
      if (!url) {
        toast.error(channel === "whatsapp" ? "El paciente no tiene WhatsApp o teléfono válido" : "El paciente no tiene correo registrado");
        return;
      }
      await officeApi.openExternalUrl(sessionToken ?? "", url);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  };

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

      <Sheet open={Boolean(editingPatient)} onOpenChange={(value) => !value && setEditingPatient(null)}>
        <SheetContent className="w-[720px] overflow-y-auto sm:max-w-[720px]">
          <SheetHeader>
            <SheetTitle>Editar paciente</SheetTitle>
          </SheetHeader>
          <div className="mt-6">
            {editingValues ? (
              <PatientForm
                initialValues={editingValues}
                onSubmit={async (values) => {
                  await updateMutation.mutateAsync(values);
                }}
                submitting={updateMutation.isPending}
                submitLabel="Actualizar paciente"
              />
            ) : null}
          </div>
        </SheetContent>
      </Sheet>

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
                <TableHead className="w-[700px]">Acciones rápidas</TableHead>
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
                        {patient.whatsapp || patient.phone ? (
                          <Button variant="outline" size="sm" onClick={() => void sendNextAppointmentMessage(patient, "whatsapp")}>
                            <MessageCircle className="h-4 w-4" />
                            WhatsApp cita
                          </Button>
                        ) : null}
                        {patient.email ? (
                          <Button variant="outline" size="sm" onClick={() => void sendNextAppointmentMessage(patient, "email")}>
                            <Mail className="h-4 w-4" />
                            Correo cita
                          </Button>
                        ) : null}
                        <Button variant="outline" size="sm" onClick={() => setEditingPatient(patient)}>
                          <Edit className="h-4 w-4" />
                          Editar
                        </Button>
                        <AlertDialog>
                          <AlertDialogTrigger asChild>
                            <Button variant="outline" size="sm">
                              <Trash2 className="h-4 w-4" />
                              Baja
                            </Button>
                          </AlertDialogTrigger>
                          <AlertDialogContent>
                            <AlertDialogHeader>
                              <AlertDialogTitle>Dar de baja a {patient.fullName}</AlertDialogTitle>
                              <AlertDialogDescription>
                                El registro quedará oculto de los listados operativos sin borrarse físicamente.
                              </AlertDialogDescription>
                            </AlertDialogHeader>
                            <AlertDialogFooter>
                              <AlertDialogCancel>Cancelar</AlertDialogCancel>
                              <AlertDialogAction onClick={() => deleteMutation.mutate(patient.id)}>Confirmar baja</AlertDialogAction>
                            </AlertDialogFooter>
                          </AlertDialogContent>
                        </AlertDialog>
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

function patientToFormValues(patient: PatientSummary): PatientFormValues {
  return {
    fullName: patient.fullName,
    birthDate: patient.birthDate ?? "",
    sex: patient.sex ?? "",
    phone: patient.phone ?? "",
    whatsapp: patient.whatsapp ?? "",
    email: patient.email ?? "",
    address: patient.address ?? "",
    emergencyContactName: patient.emergencyContactName ?? "",
    emergencyContactPhone: patient.emergencyContactPhone ?? "",
    occupation: patient.occupation ?? "",
    allergies: patient.allergies ?? "",
    systemicDiseases: patient.systemicDiseases ?? "",
    currentMedications: patient.currentMedications ?? "",
    relevantHistory: patient.relevantHistory ?? "",
    habits: patient.habits ?? "",
    generalNotes: patient.generalNotes ?? "",
  };
}

function buildAppointmentMessage(appointment: AppointmentSummary) {
  const dentistText = appointment.dentistName ? ` con ${appointment.dentistName}` : "";
  return `Hola ${appointment.patientName}, te recordamos tu cita dental el ${formatDateTime(appointment.startsAt)}${dentistText}. Motivo: ${appointment.reason}. Si necesitas reprogramar, por favor avisanos.`;
}

function buildWhatsAppUrl(appointment: AppointmentSummary) {
  const rawPhone = appointment.patientWhatsapp || appointment.patientPhone;
  const phone = rawPhone?.replace(/\D/g, "");
  if (!phone) return null;
  const normalizedPhone = phone.length === 10 ? `52${phone}` : phone;
  return `https://wa.me/${normalizedPhone}?text=${encodeURIComponent(buildAppointmentMessage(appointment))}`;
}

function buildEmailUrl(appointment: AppointmentSummary) {
  if (!appointment.patientEmail) return null;
  const subject = `Recordatorio de cita dental - ${formatDateTime(appointment.startsAt)}`;
  return `mailto:${appointment.patientEmail}?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(buildAppointmentMessage(appointment))}`;
}

async function invalidatePatientData(queryClient: ReturnType<typeof useQueryClient>) {
  await queryClient.invalidateQueries({ queryKey: ["patients"] });
  await queryClient.invalidateQueries({ queryKey: ["appointments"] });
  await queryClient.invalidateQueries({ queryKey: ["dashboard-summary"] });
  await queryClient.invalidateQueries({ queryKey: ["reports"] });
  await queryClient.invalidateQueries({ queryKey: ["global-search"] });
}
