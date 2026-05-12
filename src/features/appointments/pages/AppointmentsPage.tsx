import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ChevronLeft, ChevronRight, Plus } from "lucide-react";
import { toast } from "sonner";
import { useSearchParams } from "react-router-dom";
import { EmptyState } from "@/components/data/EmptyState";
import { PageHeader } from "@/components/data/PageHeader";
import { StatusBadge } from "@/components/data/StatusBadge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from "@/components/ui/sheet";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { AppointmentForm } from "@/features/appointments/components/AppointmentForm";
import { createAppointment, listAppointments, updateAppointmentStatus } from "@/features/appointments/services/appointment-service";
import type { AppointmentFormValues } from "@/features/appointments/schemas/appointment-schemas";
import { listUsers } from "@/features/auth/services/auth-service";
import { listPatients } from "@/features/patients/services/patient-service";
import { addDaysToDateInput, defaultAppointmentStartsAt, formatDateTime, todayInputValue } from "@/lib/api";
import { useAuthStore } from "@/store/auth-store";

const statuses = ["programada", "confirmada", "en_espera", "en_consulta", "finalizada", "cancelada", "no_asistio"];

export function AppointmentsPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const [date, setDate] = useState(() => searchParams.get("date") ?? todayInputValue());
  const [open, setOpen] = useState(false);
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

  const appointments = useQuery({
    queryKey: ["appointments", sessionToken, date],
    queryFn: () => listAppointments(sessionToken ?? "", date),
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
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const statusMutation = useMutation({
    mutationFn: ({ appointmentId, status }: { appointmentId: string; status: string }) =>
      updateAppointmentStatus(sessionToken ?? "", appointmentId, status),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["appointments"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard-summary"] });
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });

  const rows = appointments.data ?? [];

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
      </div>

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
                <TableHead className="w-[190px]">Cambiar estado</TableHead>
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
                    <Select
                      value={appointment.status}
                      onValueChange={(status) => statusMutation.mutate({ appointmentId: appointment.id, status })}
                    >
                      <SelectTrigger>
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
