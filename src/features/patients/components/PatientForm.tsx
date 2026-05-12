import { zodResolver } from "@hookform/resolvers/zod";
import { useForm, useWatch, type UseFormRegisterReturn } from "react-hook-form";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { patientSchema, type PatientFormValues } from "@/features/patients/schemas/patient-schemas";

const defaultValues: PatientFormValues = {
  fullName: "",
  birthDate: "",
  sex: "",
  phone: "",
  whatsapp: "",
  email: "",
  address: "",
  emergencyContactName: "",
  emergencyContactPhone: "",
  occupation: "",
  allergies: "",
  systemicDiseases: "",
  currentMedications: "",
  relevantHistory: "",
  habits: "",
  generalNotes: "",
};

export function PatientForm({
  onSubmit,
  submitting,
}: {
  onSubmit: (values: PatientFormValues) => Promise<void>;
  submitting?: boolean;
}) {
  const form = useForm<PatientFormValues>({ resolver: zodResolver(patientSchema), defaultValues });
  const sex = useWatch({ control: form.control, name: "sex" }) ?? "";

  return (
    <form className="grid gap-4" onSubmit={form.handleSubmit(onSubmit)}>
      <div className="grid gap-2">
        <Label htmlFor="fullName">Nombre completo</Label>
        <Input id="fullName" {...form.register("fullName")} />
        {form.formState.errors.fullName ? (
          <p className="text-xs text-destructive">{form.formState.errors.fullName.message}</p>
        ) : null}
      </div>
      <div className="grid gap-4 sm:grid-cols-[220px_1fr_1fr]">
        <Field label="Fecha de nacimiento opcional" id="birthDate" type="date" register={form.register("birthDate")} />
        <div className="grid gap-2">
          <Label>Sexo opcional</Label>
          <Select value={sex || "sin_especificar"} onValueChange={(value) => form.setValue("sex", value === "sin_especificar" ? "" : value, { shouldDirty: true })}>
            <SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="sin_especificar">Sin especificar</SelectItem>
              <SelectItem value="femenino">Femenino</SelectItem>
              <SelectItem value="masculino">Masculino</SelectItem>
              <SelectItem value="otro">Otro</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <Field label="Ocupación opcional" id="occupation" register={form.register("occupation")} />
      </div>
      <div className="grid gap-4 sm:grid-cols-3">
        <Field label="Teléfono opcional" id="phone" register={form.register("phone")} />
        <Field label="WhatsApp opcional" id="whatsapp" register={form.register("whatsapp")} />
        <Field label="Correo opcional" id="email" type="email" register={form.register("email")} />
      </div>
      <div className="grid gap-2">
        <Label htmlFor="address">Dirección opcional</Label>
        <Textarea id="address" {...form.register("address")} />
      </div>
      <div className="grid gap-4 sm:grid-cols-2">
        <Field label="Contacto de emergencia opcional" id="emergencyContactName" register={form.register("emergencyContactName")} />
        <Field label="Teléfono emergencia opcional" id="emergencyContactPhone" register={form.register("emergencyContactPhone")} />
      </div>
      <div className="grid gap-4 sm:grid-cols-2">
        <TextAreaField label="Alergias" id="allergies" register={form.register("allergies")} />
        <TextAreaField label="Enfermedades sistémicas" id="systemicDiseases" register={form.register("systemicDiseases")} />
        <TextAreaField label="Medicamentos actuales" id="currentMedications" register={form.register("currentMedications")} />
        <TextAreaField label="Antecedentes relevantes" id="relevantHistory" register={form.register("relevantHistory")} />
      </div>
      <TextAreaField label="Hábitos y notas generales" id="generalNotes" register={form.register("generalNotes")} />
      <Button type="submit" disabled={submitting || form.formState.isSubmitting}>
        Guardar paciente
      </Button>
    </form>
  );
}

function Field({
  label,
  id,
  type = "text",
  register,
}: {
  label: string;
  id: string;
  type?: string;
  register: UseFormRegisterReturn;
}) {
  return (
    <div className="grid gap-2">
      <Label htmlFor={id}>{label}</Label>
      <Input id={id} type={type} {...register} />
    </div>
  );
}

function TextAreaField({
  label,
  id,
  register,
}: {
  label: string;
  id: string;
  register: UseFormRegisterReturn;
}) {
  return (
    <div className="grid gap-2">
      <Label htmlFor={id}>{label}</Label>
      <Textarea id={id} {...register} />
    </div>
  );
}
