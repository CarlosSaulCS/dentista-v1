import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ChevronLeft, ChevronRight, Edit, Mail, MessageCircle, Plus, Search, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { useSearchParams } from "react-router-dom";
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
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from "@/components/ui/sheet";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { AppointmentForm, type AppointmentSlotSuggestion } from "@/features/appointments/components/AppointmentForm";
import {
  createAppointment,
  listAppointments,
  softDeleteAppointment,
  updateAppointment,
  updateAppointmentStatus,
} from "@/features/appointments/services/appointment-service";
import type { AppointmentFormValues } from "@/features/appointments/schemas/appointment-schemas";
import type { AppointmentSummary } from "@/features/appointments/types/appointment-types";
import { listUsers } from "@/features/auth/services/auth-service";
import { listPatients } from "@/features/patients/services/patient-service";
import { addDaysToDateInput, defaultAppointmentStartsAt, formatDateTime, todayInputValue } from "@/lib/api";
import { officeApi } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

const statuses = ["programada", "confirmada", "en_espera", "en_consulta", "finalizada", "cancelada", "no_asistio"];

export function AppointmentsPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const [date, setDate] = useState(() => searchParams.get("date") ?? todayInputValue());
  const [open, setOpen] = useState(false);
  const [statusFilter, setStatusFilter] = useState("all");
  const [search, setSearch] = useState("");
  const [editingAppointment, setEditingAppointment] = useState<AppointmentSummary | null>(null);
  const sessionToken = useAuthStore((state) => state.sessionToken);
  const queryClient = useQueryClient();
  const patientIdFromUrl = searchParams.get("patientId") ?? "";
  const newAppointmentRequested = searchParams.get("new") === "1";
  const sheetOpen = open || newAppointmentRequested;
  const initialAppointmentValues = useMemo(
    () => ({
      patientId: patientIdFromUrl,
      startsAt: defaultAppointmentStartsAt(date),
      reason: patientIdFromUrl ? "Consulta" : "",
    }),
    [date, patientIdFromUrl],
  );
  const editingValues = useMemo(
    () => (editingAppointment ? appointmentToFormValues(editingAppointment) : undefined),
    [editingAppointment],
  );

  const appointments = useQuery({
    queryKey: ["appointments", sessionToken, date, statusFilter],
    queryFn: () => listAppointments(sessionToken ?? "", date, statusFilter === "all" ? "" : statusFilter),
    enabled: Boolean(sessionToken),
  });
  const patients = useQuery({
    queryKey: ["patients", sessionToken, "appointment-picker"],
    queryFn: () => listPatients(sessionToken ?? "", "", 200),
    enabled: Boolean(sessionToken),
  });
  const users = useQuery({
    queryKey: ["users", sessionToken],
    queryFn: () => listUsers(sessionToken ?? ""),
    enabled: Boolean(sessionToken),
  });

  const createMutation = useMutation({
    mutationFn: (values: AppointmentFormValues) => createAppointment(sessionToken ?? "", values),
    onSuccess: async () => {
      toast.success("Cita creada");
      setOpen(false);
      if (newAppointmentRequested) {
        const nextParams = new URLSearchParams(searchParams);
        nextParams.delete("new");
        setSearchParams(nextParams, { replace: true });
      }
      await queryClient.invalidateQueries({ queryKey: ["appointments"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard-summary"] });
      await queryClient.invalidateQueries({ queryKey: ["reports"] });
      await queryClient.invalidateQueries({ queryKey: ["alerts"] });
      await queryClient.invalidateQueries({ queryKey: ["global-search"] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const statusMutation = useMutation({
    mutationFn: ({ appointmentId, status }: { appointmentId: string; status: string }) =>
      updateAppointmentStatus(sessionToken ?? "", appointmentId, status),
    onSuccess: async () => {
      toast.success("Estado actualizado");
      await queryClient.invalidateQueries({ queryKey: ["appointments"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard-summary"] });
      await queryClient.invalidateQueries({ queryKey: ["reports"] });
      await queryClient.invalidateQueries({ queryKey: ["alerts"] });
      await queryClient.invalidateQueries({ queryKey: ["global-search"] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const updateMutation = useMutation({
    mutationFn: (values: AppointmentFormValues) =>
      updateAppointment(sessionToken ?? "", {
        ...values,
        id: editingAppointment?.id ?? "",
        status: editingAppointment?.status ?? "programada",
      }),
    onSuccess: async () => {
      toast.success("Cita actualizada");
      setEditingAppointment(null);
      await invalidateAppointmentData(queryClient);
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const deleteMutation = useMutation({
    mutationFn: (appointmentId: string) => softDeleteAppointment(sessionToken ?? "", appointmentId),
    onSuccess: async () => {
      toast.success("Cita eliminada de la agenda");
      await invalidateAppointmentData(queryClient);
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const rows = (appointments.data ?? []).filter((appointment) => {
    const needle = search.trim().toLowerCase();
    if (!needle) return true;
    return [appointment.patientName, appointment.dentistName ?? "", appointment.reason, appointment.status]
      .join(" ")
      .toLowerCase()
      .includes(needle);
  });

  const handleOpenChange = (value: boolean) => {
    setOpen(value);
    if (!value && newAppointmentRequested) {
      const nextParams = new URLSearchParams(searchParams);
      nextParams.delete("new");
      setSearchParams(nextParams, { replace: true });
    }
  };

  const setScheduleDate = (value: string) => {
    const nextDate = value || todayInputValue();
    setDate(nextDate);
    const nextParams = new URLSearchParams(searchParams);
    nextParams.set("date", nextDate);
    setSearchParams(nextParams, { replace: true });
  };

  const findAvailableSlots = async ({
    fromDate,
    dentistUserId,
    patientId,
    durationMinutes,
    dayPart,
  }: {
    fromDate: string;
    dentistUserId: string;
    patientId: string;
    durationMinutes: number;
    dayPart: string;
  }): Promise<AppointmentSlotSuggestion[]> => {
    if (!patientId) {
      toast.error("Selecciona un paciente antes de buscar horarios");
      return [];
    }

    const duration = Math.max(5, Math.min(480, durationMinutes || 45));
    const range = dayPartRange(dayPart);
    const dentists = dentistUserId ? [dentistUserId] : (users.data ?? []).map((user) => user.id);
    const candidateDentists = dentists.length > 0 ? dentists : [""];
    const suggestions: AppointmentSlotSuggestion[] = [];

    for (let offset = 0; offset < 21 && suggestions.length < 8; offset += 1) {
      const dateValue = addDaysToDateInput(fromDate || date, offset);
      const dayAppointments = await listAppointments(sessionToken ?? "", dateValue);
      const activeAppointments = dayAppointments.filter((appointment) => !["cancelada", "finalizada", "no_asistio"].includes(appointment.status));

      for (let minutes = range.start; minutes + duration <= range.end && suggestions.length < 8; minutes += 15) {
        const slotStart = dateWithMinutes(dateValue, minutes);
        const slotEnd = new Date(slotStart.getTime() + duration * 60_000);
        if (slotStart.getTime() < Date.now() + 10 * 60_000) {
          continue;
        }

        const patientBusy = activeAppointments.some((appointment) =>
          appointment.patientId === patientId && appointmentsOverlap(slotStart, slotEnd, parseDateTime(appointment.startsAt), parseDateTime(appointment.endsAt)),
        );
        if (patientBusy) {
          continue;
        }

        const availableDentist = candidateDentists.find((candidateDentistId) => {
          if (!candidateDentistId) return true;
          return !activeAppointments.some((appointment) =>
            appointment.dentistUserId === candidateDentistId
            && appointmentsOverlap(slotStart, slotEnd, parseDateTime(appointment.startsAt), parseDateTime(appointment.endsAt)),
          );
        });
        if (availableDentist === undefined) {
          continue;
        }

        const dentistName = (users.data ?? []).find((user) => user.id === availableDentist)?.fullName ?? "Sin responsable asignado";
        suggestions.push({
          startsAt: toDateTimeLocalValue(slotStart),
          label: formatDateTime(toDateTimeLocalValue(slotStart)),
          meta: dentistName,
          dentistUserId: availableDentist || undefined,
        });
      }
    }

    if (suggestions.length === 0) {
      toast.error("No se encontraron horarios disponibles en las próximas 3 semanas");
    }
    return suggestions;
  };
  const openContactLink = async (url: string) => {
    try {
      await officeApi.openExternalUrl(sessionToken ?? "", url);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  };

  return (
    <div className="space-y-6">
      <PageHeader
        title="Agenda de citas"
        description="Vista diaria con estados operativos y trazabilidad de cambios."
        actions={
          <Sheet open={sheetOpen} onOpenChange={handleOpenChange}>
            <SheetTrigger asChild>
              <Button>
                <Plus className="h-4 w-4" />
                Nueva cita
              </Button>
            </SheetTrigger>
            <SheetContent className="w-[620px] overflow-y-auto sm:max-w-[620px]">
              <SheetHeader>
                <SheetTitle>Agendar cita</SheetTitle>
              </SheetHeader>
              <div className="mt-6">
                <AppointmentForm
                  patients={patients.data ?? []}
                  users={users.data ?? []}
                  onSubmit={async (values) => {
                    await createMutation.mutateAsync(values);
                  }}
                  submitting={createMutation.isPending}
                  initialValues={initialAppointmentValues}
                  onFindAvailableSlots={findAvailableSlots}
                />
              </div>
            </SheetContent>
          </Sheet>
        }
      />

      <div className="flex flex-wrap items-center gap-2">
        <Button variant="outline" size="icon" onClick={() => setScheduleDate(addDaysToDateInput(date, -1))} aria-label="Día anterior">
          <ChevronLeft className="h-4 w-4" />
        </Button>
        <Input className="w-[180px]" type="date" value={date} onChange={(event) => setScheduleDate(event.target.value)} />
        <Button variant="outline" size="icon" onClick={() => setScheduleDate(addDaysToDateInput(date, 1))} aria-label="Día siguiente">
          <ChevronRight className="h-4 w-4" />
        </Button>
        <Button variant="secondary" onClick={() => setScheduleDate(todayInputValue())}>
          Hoy
        </Button>
        <Select value={statusFilter} onValueChange={setStatusFilter}>
          <SelectTrigger className="w-[190px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">Todos los estados</SelectItem>
            {statuses.map((status) => (
              <SelectItem key={status} value={status}>
                {status.replaceAll("_", " ")}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <div className="flex min-w-[260px] flex-1 items-center gap-2 sm:max-w-sm">
          <Search className="h-4 w-4 text-muted-foreground" />
          <Input placeholder="Buscar paciente, motivo o estado" value={search} onChange={(event) => setSearch(event.target.value)} />
        </div>
      </div>

      <Sheet open={Boolean(editingAppointment)} onOpenChange={(value) => !value && setEditingAppointment(null)}>
        <SheetContent className="w-[620px] overflow-y-auto sm:max-w-[620px]">
          <SheetHeader>
            <SheetTitle>Editar cita</SheetTitle>
          </SheetHeader>
          <div className="mt-6">
            {editingValues ? (
              <AppointmentForm
                patients={patients.data ?? []}
                users={users.data ?? []}
                initialValues={editingValues}
                onSubmit={async (values) => {
                  await updateMutation.mutateAsync(values);
                }}
                submitting={updateMutation.isPending}
                submitLabel="Actualizar cita"
                onFindAvailableSlots={findAvailableSlots}
              />
            ) : null}
          </div>
        </SheetContent>
      </Sheet>

      {rows.length === 0 ? (
        <EmptyState title="Sin citas para esta fecha" description="Agenda una cita o cambia el filtro de fecha." />
      ) : (
        <div className="overflow-x-auto rounded-lg border bg-card">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Hora</TableHead>
                <TableHead>Paciente</TableHead>
                <TableHead>Odontólogo</TableHead>
                <TableHead>Motivo</TableHead>
                <TableHead>Estado</TableHead>
                <TableHead className="w-[520px]">Acciones</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {rows.map((appointment) => (
                <TableRow key={appointment.id}>
                  <TableCell>{formatDateTime(appointment.startsAt)}</TableCell>
                  <TableCell className="font-medium">{appointment.patientName}</TableCell>
                  <TableCell>{appointment.dentistName ?? "Sin asignar"}</TableCell>
                  <TableCell>{appointment.reason}</TableCell>
                  <TableCell>
                    <StatusBadge status={appointment.status} />
                  </TableCell>
                  <TableCell>
                    <div className="flex flex-wrap items-center gap-2">
                      <Select
                        value={appointment.status}
                        onValueChange={(status) => statusMutation.mutate({ appointmentId: appointment.id, status })}
                      >
                        <SelectTrigger className="w-[170px]">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          {statuses.map((status) => (
                            <SelectItem key={status} value={status}>
                              {status.replaceAll("_", " ")}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                      <Button variant="outline" size="sm" onClick={() => setEditingAppointment(appointment)}>
                        <Edit className="h-4 w-4" />
                        Editar
                      </Button>
                      {buildWhatsAppUrl(appointment) ? (
                        <Button variant="outline" size="sm" onClick={() => void openContactLink(buildWhatsAppUrl(appointment) ?? "")}>
                          <MessageCircle className="h-4 w-4" />
                          WhatsApp
                        </Button>
                      ) : null}
                      {buildEmailUrl(appointment) ? (
                        <Button variant="outline" size="sm" onClick={() => void openContactLink(buildEmailUrl(appointment) ?? "")}>
                          <Mail className="h-4 w-4" />
                          Correo
                        </Button>
                      ) : null}
                      <AlertDialog>
                        <AlertDialogTrigger asChild>
                          <Button variant="outline" size="sm">
                            <Trash2 className="h-4 w-4" />
                            Eliminar
                          </Button>
                        </AlertDialogTrigger>
                        <AlertDialogContent>
                          <AlertDialogHeader>
                            <AlertDialogTitle>Eliminar cita de {appointment.patientName}</AlertDialogTitle>
                            <AlertDialogDescription>
                              La cita se cancelará y quedará oculta de la agenda, conservando auditoría.
                            </AlertDialogDescription>
                          </AlertDialogHeader>
                          <AlertDialogFooter>
                            <AlertDialogCancel>Cancelar</AlertDialogCancel>
                            <AlertDialogAction onClick={() => deleteMutation.mutate(appointment.id)}>Eliminar cita</AlertDialogAction>
                          </AlertDialogFooter>
                        </AlertDialogContent>
                      </AlertDialog>
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  );
}

function dayPartRange(dayPart: string) {
  if (dayPart === "morning") return { start: 9 * 60, end: 13 * 60 };
  if (dayPart === "afternoon") return { start: 13 * 60, end: 18 * 60 };
  return { start: 9 * 60, end: 18 * 60 };
}

function dateWithMinutes(dateValue: string, minutes: number) {
  const [year, month, day] = dateValue.split("-").map(Number);
  const date = new Date(year, month - 1, day, 0, 0, 0, 0);
  date.setMinutes(minutes);
  return date;
}

function parseDateTime(value: string) {
  return new Date(value);
}

function appointmentsOverlap(start: Date, end: Date, busyStart: Date, busyEnd: Date) {
  return start.getTime() < busyEnd.getTime() && end.getTime() > busyStart.getTime();
}

function toDateTimeLocalValue(date: Date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  const hours = String(date.getHours()).padStart(2, "0");
  const minutes = String(date.getMinutes()).padStart(2, "0");
  return `${year}-${month}-${day}T${hours}:${minutes}`;
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

function appointmentToFormValues(appointment: AppointmentSummary): AppointmentFormValues {
  return {
    patientId: appointment.patientId,
    dentistUserId: appointment.dentistUserId ?? "",
    startsAt: appointment.startsAt.slice(0, 16),
    durationMinutes: appointment.durationMinutes,
    reason: appointment.reason,
    appointmentType: appointment.appointmentType,
    notes: appointment.notes ?? "",
  };
}

async function invalidateAppointmentData(queryClient: ReturnType<typeof useQueryClient>) {
  await queryClient.invalidateQueries({ queryKey: ["appointments"] });
  await queryClient.invalidateQueries({ queryKey: ["dashboard-summary"] });
  await queryClient.invalidateQueries({ queryKey: ["reports"] });
  await queryClient.invalidateQueries({ queryKey: ["alerts"] });
  await queryClient.invalidateQueries({ queryKey: ["global-search"] });
}
