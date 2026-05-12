import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { OdontogramEntry } from "@/features/odontogram/types/odontogram-types";

const permanentTeeth = [
  "18",
  "17",
  "16",
  "15",
  "14",
  "13",
  "12",
  "11",
  "21",
  "22",
  "23",
  "24",
  "25",
  "26",
  "27",
  "28",
  "48",
  "47",
  "46",
  "45",
  "44",
  "43",
  "42",
  "41",
  "31",
  "32",
  "33",
  "34",
  "35",
  "36",
  "37",
  "38",
];

const stateClasses: Record<string, string> = {
  sano: "border-emerald-300 bg-emerald-50 text-emerald-700",
  caries: "border-red-300 bg-red-50 text-red-700",
  restauracion: "border-sky-300 bg-sky-50 text-sky-700",
  ausente: "border-slate-300 bg-slate-100 text-slate-500",
  tratamiento_pendiente: "border-amber-300 bg-amber-50 text-amber-700",
  tratamiento_realizado: "border-blue-300 bg-blue-50 text-blue-700",
};

export function OdontogramGrid({
  entries,
  selectedTooth,
  onSelect,
}: {
  entries: OdontogramEntry[];
  selectedTooth: string;
  onSelect: (tooth: string) => void;
}) {
  const entryByTooth = new Map(entries.map((entry) => [entry.toothNumber, entry]));

  return (
    <div className="grid grid-cols-8 gap-2 xl:grid-cols-[repeat(16,minmax(0,1fr))]">
      {permanentTeeth.map((tooth) => {
        const entry = entryByTooth.get(tooth);
        return (
          <Button
            key={tooth}
            type="button"
            variant="outline"
            className={cn(
              "h-16 flex-col gap-1 border bg-card",
              entry && stateClasses[entry.state],
              selectedTooth === tooth && "ring-2 ring-primary ring-offset-2",
            )}
            onClick={() => onSelect(tooth)}
          >
            <span className="text-base font-semibold">{tooth}</span>
            <span className="max-w-full truncate text-[10px]">{entry?.state?.replaceAll("_", " ") ?? "sin registro"}</span>
          </Button>
        );
      })}
    </div>
  );
}
