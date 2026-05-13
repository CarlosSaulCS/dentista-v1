import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Bell, CalendarCheck, CalendarClock, CircleDollarSign, FileText, Package, Search, Users } from "lucide-react";
import { Link } from "react-router-dom";
import { Area, AreaChart, Bar, BarChart, CartesianGrid, Cell, Legend, Pie, PieChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { PageHeader } from "@/components/data/PageHeader";
import { StatusBadge } from "@/components/data/StatusBadge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { getDashboardSummary } from "@/features/dashboard/services/dashboard-service";
import { formatCurrency, formatDateTime } from "@/lib/api";
import { officeApi } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

const chartPalette = ["#2563eb", "#0f766e", "#f59e0b", "#dc2626", "#7c3aed", "#0891b2"];
const statusColors: Record<string, string> = {
  programada: "#f59e0b",
  confirmada: "#059669",
  en_espera: "#0891b2",
  en_consulta: "#2563eb",
  finalizada: "#475569",
  cancelada: "#dc2626",
  no_asistio: "#9333ea",
};

export function DashboardPage() {
  const [globalSearch, setGlobalSearch] = useState("");
  const sessionToken = useAuthStore((state) => state.sessionToken);
  const { data, isLoading } = useQuery({
    queryKey: ["dashboard-summary", sessionToken],
    queryFn: () => getDashboardSummary(sessionToken ?? ""),
    enabled: Boolean(sessionToken),
  });
  const searchResults = useQuery({
    queryKey: ["global-search", sessionToken, globalSearch],
    queryFn: () => officeApi.globalSearch(sessionToken ?? "", globalSearch),
    enabled: Boolean(sessionToken) && globalSearch.trim().length >= 2,
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
  const appointmentStatusSeries = data.appointmentStatuses.map((point, index) => ({
    ...point,
    label: formatChartLabel(point.label),
    fill: statusColors[point.label] ?? chartPalette[index % chartPalette.length],
  }));
  const paymentMethods = data.paymentMethods.map((point, index) => ({
    label: formatChartLabel(point.label),
    value: point.value / 100,
    fill: chartPalette[index % chartPalette.length],
  }));

  return (
    <div className="space-y-6">
      <PageHeader title="Dashboard" description="Agenda, ingresos, pendientes y alertas clínicas." />

      <div className="grid gap-3">
        <div className="flex max-w-xl items-center gap-2">
          <Search className="h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Buscar pacientes, citas, insumos o presupuestos"
            value={globalSearch}
            onChange={(event) => setGlobalSearch(event.target.value)}
          />
        </div>
        {globalSearch.trim().length >= 2 ? (
          <Card>
            <CardContent className="grid gap-2 p-3">
              {(searchResults.data ?? []).length === 0 && !searchResults.isFetching ? (
                <p className="px-1 py-2 text-sm text-muted-foreground">Sin resultados para la búsqueda.</p>
              ) : (
                (searchResults.data ?? []).map((result) => (
                  <Link key={`${result.entityType}-${result.id}`} to={result.route} className="flex items-center justify-between gap-3 rounded-md px-3 py-2 text-sm hover:bg-muted">
                    <span className="min-w-0">
                      <span className="block truncate font-medium">{result.title}</span>
                      <span className="block truncate text-xs text-muted-foreground">{result.subtitle}</span>
                    </span>
                    {result.status ? <StatusBadge status={result.status} /> : null}
                  </Link>
                ))
              )}
            </CardContent>
          </Card>
        ) : null}
      </div>

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
        <Card className="overflow-hidden border-blue-100 bg-gradient-to-b from-white to-blue-50/40">
          <CardHeader>
            <CardTitle>Ingresos últimos 7 días</CardTitle>
          </CardHeader>
          <CardContent className="h-72">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={incomeSeries} margin={{ left: 4, right: 10, top: 8, bottom: 0 }}>
                <defs>
                  <linearGradient id="incomeFill" x1="0" x2="0" y1="0" y2="1">
                    <stop offset="5%" stopColor="#2563eb" stopOpacity={0.35} />
                    <stop offset="95%" stopColor="#2563eb" stopOpacity={0.04} />
                  </linearGradient>
                </defs>
                <CartesianGrid stroke="#dbeafe" strokeDasharray="4 4" vertical={false} />
                <XAxis dataKey="label" tickLine={false} axisLine={false} tickMargin={10} />
                <YAxis tickLine={false} axisLine={false} tickFormatter={(value) => `$${value}`} width={58} />
                <Tooltip contentStyle={{ borderRadius: 8, borderColor: "#bfdbfe" }} formatter={(value) => formatCurrency(Number(value) * 100)} />
                <Area type="monotone" dataKey="value" stroke="#2563eb" strokeWidth={3} fill="url(#incomeFill)" activeDot={{ r: 5 }} />
              </AreaChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        <Card className="overflow-hidden border-emerald-100 bg-gradient-to-b from-white to-emerald-50/40">
          <CardHeader>
            <CardTitle>Citas de hoy por estado</CardTitle>
          </CardHeader>
          <CardContent className="h-72">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={appointmentStatusSeries} margin={{ left: 2, right: 8, top: 8, bottom: 0 }}>
                <CartesianGrid stroke="#d1fae5" strokeDasharray="4 4" vertical={false} />
                <XAxis dataKey="label" tickLine={false} axisLine={false} tickMargin={10} />
                <YAxis allowDecimals={false} tickLine={false} axisLine={false} width={34} />
                <Tooltip contentStyle={{ borderRadius: 8, borderColor: "#bbf7d0" }} />
                <Bar dataKey="value" radius={[6, 6, 0, 0]}>
                  {appointmentStatusSeries.map((point) => (
                    <Cell key={point.label} fill={point.fill} />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        <Card className="overflow-hidden border-cyan-100 bg-gradient-to-b from-white to-cyan-50/40">
          <CardHeader>
            <CardTitle>Pagos del mes por método</CardTitle>
          </CardHeader>
          <CardContent className="h-72">
            <ResponsiveContainer width="100%" height="100%">
              <PieChart>
                <Tooltip contentStyle={{ borderRadius: 8, borderColor: "#bae6fd" }} formatter={(value) => formatCurrency(Number(value) * 100)} />
                <Pie data={paymentMethods} dataKey="value" innerRadius={52} outerRadius={88} paddingAngle={3} nameKey="label">
                  {paymentMethods.map((point) => (
                    <Cell key={point.label} fill={point.fill} />
                  ))}
                </Pie>
                <Legend iconType="circle" layout="horizontal" verticalAlign="bottom" />
              </PieChart>
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

function formatChartLabel(value: string) {
  return value.replaceAll("_", " ");
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
