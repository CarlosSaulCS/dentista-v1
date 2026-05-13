import { useEffect, useState } from "react";
import { zodResolver } from "@hookform/resolvers/zod";
import { CalendarSearch } from "lucide-react";
import { useForm, useWatch } from "react-hook-form";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { appointmentSchema, type AppointmentFormValues } from "@/features/appointments/schemas/appointment-schemas";
import { todayInputValue } from "@/lib/api";
import type { PatientSummary } from "@/features/patients/types/patient-types";
import type { UserListItem } from "@/types/shared";

const appointmentTypes = ["primera_valoracion", "limpieza", "revision", "urgencia", "tratamiento", "seguimiento", "ortodoncia", "cirugia", "otro"];
const appointmentReasonByType: Record<string, string> = {
  primera_valoracion: "Primera valoración",
  limpieza: "Limpieza dental",
  revision: "Revisión",
  urgencia: "Urgencia dental",
  tratamiento: "Tratamiento dental",
  seguimiento: "Seguimiento",
  ortodoncia: "Control de ortodoncia",
  cirugia: "Cirugía dental",
  otro: "Consulta",
};

const defaultAppointmentValues: AppointmentFormValues = {
  patientId: "",
  dentistUserId: "",
  startsAt: "",
  durationMinutes: 45,
  reason: "",
  appointmentType: "revision",
  notes: "",
};

export type AppointmentSlotSuggestion = {
  startsAt: string;
  label: string;
  meta: string;
  dentistUserId?: string;
};

export function AppointmentForm({
  patients,
  users,
  onSubmit,
  submitting,
  initialValues,
  submitLabel = "Guardar cita",
  onFindAvailableSlots,
}: {
  patients: PatientSummary[];
  users: UserListItem[];
  onSubmit: (values: AppointmentFormValues) => Promise<void>;
  submitting?: boolean;
  initialValues?: Partial<AppointmentFormValues>;
  submitLabel?: string;
  onFindAvailableSlots?: (params: {
    fromDate: string;
    dentistUserId: string;
    patientId: string;
    durationMinutes: number;
    dayPart: string;
  }) => Promise<AppointmentSlotSuggestion[]>;
}) {
  const [fromDate, setFromDate] = useState(() => initialValues?.startsAt?.slice(0, 10) || todayInputValue());
  const [dayPart, setDayPart] = useState("any");
  const [slots, setSlots] = useState<AppointmentSlotSuggestion[]>([]);
  const [findingSlots, setFindingSlots] = useState(false);
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
  const durationMinutes = useWatch({ control: form.control, name: "durationMinutes" });
  const selectedPatient = patients.find((patient) => patient.id === patientId);

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

  useEffect(() => {
    const currentReason = form.getValues("reason").trim();
    if (!currentReason || Object.values(appointmentReasonByType).includes(currentReason)) {
      form.setValue("reason", appointmentReasonByType[appointmentType] ?? "Consulta", { shouldDirty: true });
    }
  }, [appointmentType, form]);

  const findSlots = async () => {
    if (!onFindAvailableSlots) return;
    setFindingSlots(true);
    try {
      const suggestions = await onFindAvailableSlots({
        fromDate,
        dentistUserId: dentistUserId ?? "",
        patientId: patientId ?? "",
        durationMinutes: Number(durationMinutes || defaultAppointmentValues.durationMinutes),
        dayPart,
      });
      setSlots(suggestions);
    } finally {
      setFindingSlots(false);
    }
  };

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
        {selectedPatient ? (
          <div className="rounded-md border bg-muted/35 px-3 py-2 text-xs text-muted-foreground">
            <span className="font-medium text-foreground">{selectedPatient.fullName}</span>
            <span className="ml-2">{selectedPatient.phone || selectedPatient.whatsapp || selectedPatient.email || "Sin contacto"}</span>
            {selectedPatient.allergies ? <span className="ml-2">Alergias: {selectedPatient.allergies}</span> : null}
          </div>
        ) : null}
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
      {onFindAvailableSlots ? (
        <div className="grid gap-3 rounded-lg border bg-slate-50 p-3">
          <div className="flex items-center gap-2 text-sm font-medium">
            <CalendarSearch className="h-4 w-4 text-primary" />
            Agenda rápida
          </div>
          <div className="grid gap-3 sm:grid-cols-[1fr_1fr_auto] sm:items-end">
            <div className="grid gap-2">
              <Label htmlFor="slot-from-date">Desde</Label>
              <Input id="slot-from-date" type="date" value={fromDate} onChange={(event) => setFromDate(event.target.value)} />
            </div>
            <div className="grid gap-2">
              <Label>Preferencia</Label>
              <Select value={dayPart} onValueChange={setDayPart}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="any">Cualquier horario</SelectItem>
                  <SelectItem value="morning">Mañana</SelectItem>
                  <SelectItem value="afternoon">Tarde</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <Button type="button" variant="outline" onClick={() => void findSlots()} disabled={findingSlots}>
              Buscar horarios
            </Button>
          </div>
          {slots.length > 0 ? (
            <div className="grid gap-2 sm:grid-cols-2">
              {slots.map((slot) => (
                <Button
                  key={slot.startsAt}
                  type="button"
                  variant="secondary"
                  className="h-auto justify-start px-3 py-2 text-left"
                  onClick={() => {
                    form.setValue("startsAt", slot.startsAt, { shouldDirty: true, shouldValidate: true });
                    if (slot.dentistUserId) {
                      form.setValue("dentistUserId", slot.dentistUserId, { shouldDirty: true, shouldValidate: true });
                    }
                    setSlots([]);
                  }}
                >
                  <span>
                    <span className="block font-medium">{slot.label}</span>
                    <span className="block text-xs text-muted-foreground">{slot.meta}</span>
                  </span>
                </Button>
              ))}
            </div>
          ) : null}
        </div>
      ) : null}
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
        {submitLabel}
      </Button>
    </form>
  );
}
