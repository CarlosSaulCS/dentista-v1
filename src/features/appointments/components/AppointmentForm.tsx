import { useEffect } from "react";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm, useWatch } from "react-hook-form";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { appointmentSchema, type AppointmentFormValues } from "@/features/appointments/schemas/appointment-schemas";
import type { PatientSummary } from "@/features/patients/types/patient-types";
import type { UserListItem } from "@/types/shared";

const appointmentTypes = ["primera_valoracion", "limpieza", "revision", "urgencia", "tratamiento", "seguimiento", "ortodoncia", "cirugia", "otro"];

const defaultAppointmentValues: AppointmentFormValues = {
  patientId: "",
  dentistUserId: "",
  startsAt: "",
  durationMinutes: 45,
  reason: "",
  appointmentType: "revision",
  notes: "",
};

export function AppointmentForm({
  patients,
  users,
  onSubmit,
  submitting,
  initialValues,
}: {
  patients: PatientSummary[];
  users: UserListItem[];
  onSubmit: (values: AppointmentFormValues) => Promise<void>;
  submitting?: boolean;
  initialValues?: Partial<AppointmentFormValues>;
}) {
  const initialPatientId = initialValues?.patientId ?? defaultAppointmentValues.patientId;
  const initialDentistUserId = initialValues?.dentistUserId ?? defaultAppointmentValues.dentistUserId;
  const initialStartsAt = initialValues?.startsAt ?? defaultAppointmentValues.startsAt;
  const initialDurationMinutes = initialValues?.durationMinutes ?? defaultAppointmentValues.durationMinutes;
  const initialReason = initialValues?.reason ?? defaultAppointmentValues.reason;
  const initialAppointmentType = initialValues?.appointmentType ?? defaultAppointmentValues.appointmentType;
  const initialNotes = initialValues?.notes ?? defaultAppointmentValues.notes;
  const form = useForm<AppointmentFormValues>({
    resolver: zodResolver(appointmentSchema),
    defaultValues: {
      patientId: initialPatientId,
      dentistUserId: initialDentistUserId,
      startsAt: initialStartsAt,
      durationMinutes: initialDurationMinutes,
      reason: initialReason,
      appointmentType: initialAppointmentType,
      notes: initialNotes,
    },
  });
  const patientId = useWatch({ control: form.control, name: "patientId" });
  const dentistUserId = useWatch({ control: form.control, name: "dentistUserId" });
  const appointmentType = useWatch({ control: form.control, name: "appointmentType" });

  useEffect(() => {
    form.reset({
      patientId: initialPatientId,
      dentistUserId: initialDentistUserId,
      startsAt: initialStartsAt,
      durationMinutes: initialDurationMinutes,
      reason: initialReason,
      appointmentType: initialAppointmentType,
      notes: initialNotes,
    });
  }, [
    form,
    initialAppointmentType,
    initialDentistUserId,
    initialDurationMinutes,
    initialNotes,
    initialPatientId,
    initialReason,
    initialStartsAt,
  ]);

  return (
    <form className="grid gap-4" onSubmit={form.handleSubmit(onSubmit)}>
      <div className="grid gap-2">
        <Label>Paciente</Label>
        <Select value={patientId} onValueChange={(value) => form.setValue("patientId", value, { shouldValidate: true })}>
          <SelectTrigger>
            <SelectValue placeholder="Selecciona paciente" />
          </SelectTrigger>
          <SelectContent>
            {patients.map((patient) => (
              <SelectItem key={patient.id} value={patient.id}>
                {patient.fullName}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <div className="grid gap-2">
        <Label>Odontólogo</Label>
        <Select value={dentistUserId} onValueChange={(value) => form.setValue("dentistUserId", value)}>
          <SelectTrigger>
            <SelectValue placeholder="Selecciona responsable" />
          </SelectTrigger>
          <SelectContent>
            {users.map((user) => (
              <SelectItem key={user.id} value={user.id}>
                {user.fullName}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="grid gap-2">
          <Label htmlFor="startsAt">Fecha y hora</Label>
          <Input id="startsAt" type="datetime-local" {...form.register("startsAt")} />
        </div>
        <div className="grid gap-2">
          <Label htmlFor="durationMinutes">Duración</Label>
          <Input id="durationMinutes" type="number" min={5} step={5} {...form.register("durationMinutes", { valueAsNumber: true })} />
        </div>
      </div>
      <div className="grid gap-2">
        <Label>Tipo de cita</Label>
        <Select value={appointmentType} onValueChange={(value) => form.setValue("appointmentType", value, { shouldValidate: true })}>
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {appointmentTypes.map((type) => (
              <SelectItem key={type} value={type}>
                {type.replaceAll("_", " ")}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <div className="grid gap-2">
        <Label htmlFor="reason">Motivo</Label>
        <Input id="reason" {...form.register("reason")} />
      </div>
      <div className="grid gap-2">
        <Label htmlFor="notes">Notas</Label>
        <Textarea id="notes" {...form.register("notes")} />
      </div>
      <Button type="submit" disabled={submitting || form.formState.isSubmitting}>
        Guardar cita
      </Button>
    </form>
  );
}
