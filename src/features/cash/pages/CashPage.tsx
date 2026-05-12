import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { PageHeader } from "@/components/data/PageHeader";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { formatCurrency, formatDateTime } from "@/lib/api";
import { officeApi } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

export function CashPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken) ?? "";
  const queryClient = useQueryClient();
  const [opening, setOpening] = useState("");
  const [counted, setCounted] = useState("");
  const cash = useQuery({ queryKey: ["cash-register", sessionToken], queryFn: () => officeApi.getCurrentCashRegister(sessionToken), enabled: Boolean(sessionToken) });
  const openMutation = useMutation({
    mutationFn: () => officeApi.openCashRegister(sessionToken, Math.round(Number(opening || 0) * 100)),
    onSuccess: async () => { toast.success("Caja abierta"); await queryClient.invalidateQueries({ queryKey: ["cash-register"] }); },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });
  const closeMutation = useMutation({
    mutationFn: () => officeApi.closeCashRegister(sessionToken, cash.data?.id ?? "", Math.round(Number(counted || 0) * 100)),
    onSuccess: async () => { toast.success("Caja cerrada"); await queryClient.invalidateQueries({ queryKey: ["cash-register"] }); },
    onError: (error) => toast.error(error instanceof Error ? error.message : String(error)),
  });
  const total = (cash.data?.totalCashCents ?? 0) + (cash.data?.totalTransferCents ?? 0) + (cash.data?.totalCardCents ?? 0) + (cash.data?.totalOtherCents ?? 0);
  return <div className="space-y-6">
    <PageHeader title="Caja" description="Apertura, pagos vinculados y cierre de corte local." />
    {!cash.data ? <Card><CardHeader><CardTitle>Abrir caja</CardTitle></CardHeader><CardContent className="grid max-w-sm gap-3"><Label>Fondo inicial MXN</Label><Input type="number" value={opening} onChange={(event) => setOpening(event.target.value)} /><Button onClick={() => openMutation.mutate()} disabled={openMutation.isPending}>Abrir caja</Button></CardContent></Card> :
      <div className="grid gap-4 md:grid-cols-4">
        <Metric title="Fondo inicial" value={formatCurrency(cash.data.openingFloatCents)} />
        <Metric title="Efectivo" value={formatCurrency(cash.data.totalCashCents)} />
        <Metric title="Transferencia" value={formatCurrency(cash.data.totalTransferCents)} />
        <Metric title="Tarjeta/otros" value={formatCurrency(cash.data.totalCardCents + cash.data.totalOtherCents)} />
        <Card className="md:col-span-4"><CardContent className="grid gap-3 p-5 md:grid-cols-[1fr_240px_160px] md:items-end">
          <div><div className="text-sm text-muted-foreground">Caja abierta por {cash.data.openedByName}</div><div className="font-medium">{formatDateTime(cash.data.openedAt)} · Total entradas {formatCurrency(total)}</div></div>
          <div className="grid gap-2"><Label>Efectivo contado MXN</Label><Input type="number" value={counted} onChange={(event) => setCounted(event.target.value)} /></div>
          <Button onClick={() => closeMutation.mutate()} disabled={closeMutation.isPending}>Cerrar caja</Button>
        </CardContent></Card>
      </div>}
  </div>;
}

function Metric({ title, value }: { title: string; value: string }) {
  return <Card><CardContent className="p-5"><p className="text-sm text-muted-foreground">{title}</p><p className="mt-2 text-2xl font-semibold">{value}</p></CardContent></Card>;
}
