import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

const toneByStatus: Record<string, string> = {
  active: "border-emerald-200 bg-emerald-50 text-emerald-700",
  programada: "border-sky-200 bg-sky-50 text-sky-700",
  confirmada: "border-emerald-200 bg-emerald-50 text-emerald-700",
  en_espera: "border-amber-200 bg-amber-50 text-amber-700",
  en_consulta: "border-blue-200 bg-blue-50 text-blue-700",
  finalizada: "border-slate-200 bg-slate-50 text-slate-700",
  cancelada: "border-red-200 bg-red-50 text-red-700",
  no_asistio: "border-red-200 bg-red-50 text-red-700",
  open: "border-amber-200 bg-amber-50 text-amber-700",
  resolved: "border-emerald-200 bg-emerald-50 text-emerald-700",
  critica: "border-red-200 bg-red-50 text-red-700",
  alta: "border-orange-200 bg-orange-50 text-orange-700",
  media: "border-amber-200 bg-amber-50 text-amber-700",
  baja: "border-slate-200 bg-slate-50 text-slate-700",
};

export function StatusBadge({ status }: { status: string }) {
  return (
    <Badge variant="outline" className={cn("capitalize", toneByStatus[status] ?? "bg-muted text-muted-foreground")}>
      {status.replaceAll("_", " ")}
    </Badge>
  );
}
