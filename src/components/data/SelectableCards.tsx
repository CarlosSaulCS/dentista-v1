import { Check } from "lucide-react";
import { cn } from "@/lib/utils";

export type SelectableCardOption = {
  value: string;
  label: string;
  description?: string;
};

export function SelectableCards({
  value,
  options,
  onChange,
  columns = "sm:grid-cols-3",
}: {
  value: string;
  options: SelectableCardOption[];
  onChange: (value: string) => void;
  columns?: string;
}) {
  return (
    <div className={cn("grid gap-2", columns)}>
      {options.map((option) => {
        const selected = value === option.value;
        return (
          <button
            key={option.value}
            type="button"
            onClick={() => onChange(option.value)}
            className={cn(
              "flex min-h-[68px] items-start justify-between gap-3 rounded-lg border bg-card p-3 text-left text-sm transition hover:border-primary/50 hover:bg-primary/5",
              selected && "border-primary bg-primary/10 ring-1 ring-primary/30",
            )}
          >
            <span>
              <span className="block font-medium text-foreground">{option.label}</span>
              {option.description ? (
                <span className="mt-1 block text-xs leading-5 text-muted-foreground">{option.description}</span>
              ) : null}
            </span>
            {selected ? <Check className="mt-0.5 h-4 w-4 shrink-0 text-primary" /> : null}
          </button>
        );
      })}
    </div>
  );
}
