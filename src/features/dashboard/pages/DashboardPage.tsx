import { useQuery } from "@tanstack/react-query";
import { Bell, CalendarCheck, CalendarClock, CircleDollarSign, FileText, Package, Users } from "lucide-react";
import { Link } from "react-router-dom";
import { Bar, BarChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { PageHeader } from "@/components/data/PageHeader";
import { StatusBadge } from "@/components/data/StatusBadge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { getDashboardSummary } from "@/features/dashboard/services/dashboard-service";
import { formatCurrency, formatDateTime } from "@/lib/api";
import { useAuthStore } from "@/store/auth-store";

export function DashboardPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken);
  const { data, isLoading } = useQuery({
    queryKey: ["dashboard-summary", sessionToken],
    queryFn: () => getDashboardSummary(sessionToken ?? ""),
    enabled: Boolean(sessionToken),
  });

  if (isLoading || !data) {
    return (
      <div className="space-y-6">
        <PageHeader title="Dashboard" description="Resumen operativo del consultorio" />
        <div className="grid gap-4 md:grid-cols-4">
          {Array.from({ length: 8 }).map((_, index) => (
            <Skeleton key={index} className="h-28" />
          ))}
        </div>
      </div>
    );
  }

  const incomeSeries = data.incomeSeries.map((point) => ({ label: point.label.slice(5), value: point.value / 100 }));
  const paymentMethods = data.paymentMethods.map((point) => ({ label: point.label, value: point.value / 100 }));

  return (
    <div className="space-y-6">
      <PageHeader title="Dashboard" description="Agenda, ingresos, pendientes y alertas clínicas." />

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard title="Citas de hoy" value={data.appointmentsToday} icon={CalendarClock} to="/appointments" />
        <MetricCard title="Confirmadas" value={data.confirmedToday} icon={CalendarCheck} tone="text-emerald-600" to="/appointments" />
        <MetricCard title="Ingresos del día" value={formatCurrency(data.revenueTodayCents)} icon={CircleDollarSign} to="/payments" />
        <MetricCard title="Pacientes nuevos" value={data.newPatientsMonth} icon={Users} to="/patients" />
        <MetricCard title="Presupuestos pendientes" value={data.pendingEstimates} icon={FileText} to="/estimates" />
        <MetricCard title="Planes activos" value={data.activeTreatmentPlans} icon={FileText} to="/treatment-plans" />
        <MetricCard title="Inventario bajo" value={data.lowInventory} icon={Package} tone="text-amber-600" to="/inventory" />
        <MetricCard title="Alertas abiertas" value={data.openAlerts} icon={Bell} tone="text-red-600" to="/alerts" />
      </div>

      <div className="grid gap-4 xl:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle>Ingresos últimos 7 días</CardTitle>
          </CardHeader>
          <CardContent className="h-72">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={incomeSeries}>
                <CartesianGrid strokeDasharray="3 3" vertical={false} />
                <XAxis dataKey="label" />
                <YAxis />
                <Tooltip formatter={(value) => formatCurrency(Number(value) * 100)} />
                <Bar dataKey="value" fill="#2563eb" radius={[4, 4, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Citas de hoy por estado</CardTitle>
          </CardHeader>
          <CardContent className="h-72">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={data.appointmentStatuses}>
                <CartesianGrid strokeDasharray="3 3" vertical={false} />
                <XAxis dataKey="label" />
                <YAxis allowDecimals={false} />
                <Tooltip />
                <Bar dataKey="value" fill="#16a34a" radius={[4, 4, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Pagos del mes por método</CardTitle>
          </CardHeader>
          <CardContent className="h-72">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={paymentMethods}>
                <CartesianGrid strokeDasharray="3 3" vertical={false} />
                <XAxis dataKey="label" />
                <YAxis />
                <Tooltip formatter={(value) => formatCurrency(Number(value) * 100)} />
                <Bar dataKey="value" fill="#0ea5e9" radius={[4, 4, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 xl:grid-cols-[1.1fr_0.9fr]">
        <Card>
          <CardHeader>
            <CardTitle>Próximas citas</CardTitle>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Paciente</TableHead>
                  <TableHead>Fecha</TableHead>
                  <TableHead>Motivo</TableHead>
                  <TableHead>Estado</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {data.upcomingAppointments.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={4} className="h-24 text-center text-muted-foreground">
                      No hay citas próximas registradas.
                    </TableCell>
                  </TableRow>
                ) : (
                  data.upcomingAppointments.map((appointment) => (
                    <TableRow key={appointment.id}>
                      <TableCell className="font-medium">{appointment.patientName}</TableCell>
                      <TableCell>{formatDateTime(appointment.startsAt)}</TableCell>
                      <TableCell>{appointment.reason}</TableCell>
                      <TableCell>
                        <StatusBadge status={appointment.status} />
                      </TableCell>
                    </TableRow>
                  ))
                )}
              </TableBody>
            </Table>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Alertas críticas y altas</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {data.criticalAlerts.length === 0 ? (
              <p className="text-sm text-muted-foreground">Sin alertas críticas abiertas.</p>
            ) : (
              data.criticalAlerts.map((alert) => (
                <div key={alert.id} className="rounded-md border p-3">
                  <div className="flex items-center justify-between gap-3">
                    <div className="font-medium">{alert.title}</div>
                    <StatusBadge status={alert.priority} />
                  </div>
                  <p className="mt-1 text-sm text-muted-foreground">{alert.message}</p>
                </div>
              ))
            )}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Resurtido recomendado</CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Insumo</TableHead>
                <TableHead>Existencia</TableHead>
                <TableHead>Mínimo</TableHead>
                <TableHead>Sugerido</TableHead>
                <TableHead>Costo estimado</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.restockItems.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={5} className="h-24 text-center text-muted-foreground">
                    El inventario no requiere resurtido en este momento.
                  </TableCell>
                </TableRow>
              ) : (
                data.restockItems.slice(0, 6).map((item) => (
                  <TableRow key={item.id}>
                    <TableCell className="font-medium">{item.name}</TableCell>
                    <TableCell>{item.currentQuantity} {item.unit}</TableCell>
                    <TableCell>{item.minimumStock} {item.unit}</TableCell>
                    <TableCell>{item.suggestedQuantity} {item.unit}</TableCell>
                    <TableCell>{formatCurrency(item.estimatedCostCents)}</TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
      </div>
  );
}

function MetricCard({
  title,
  value,
  icon: Icon,
  tone = "text-primary",
  to,
}: {
  title: string;
  value: string | number;
  icon: typeof CalendarClock;
  tone?: string;
  to?: string;
}) {
  const card = (
    <Card>
      <CardContent className="flex items-center justify-between p-5">
        <div>
          <p className="text-sm text-muted-foreground">{title}</p>
          <p className="mt-2 text-2xl font-semibold">{value}</p>
        </div>
        <Icon className={`h-5 w-5 ${tone}`} />
      </CardContent>
    </Card>
  );

  if (!to) {
    return card;
  }

  return (
    <Link className="block rounded-lg transition hover:-translate-y-0.5 hover:shadow-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" to={to}>
      {card}
    </Link>
  );
}
