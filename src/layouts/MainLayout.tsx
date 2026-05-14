import { useState } from "react";
import { NavLink, Outlet, useNavigate } from "react-router-dom";
import {
  Activity,
  Archive,
  Bell,
  Building2,
  CalendarDays,
  ChartNoAxesColumn,
  ChevronsLeft,
  ChevronsRight,
  CircleDollarSign,
  ClipboardPlus,
  FileClock,
  Files,
  FileText,
  LayoutDashboard,
  LogOut,
  Package,
  ScrollText,
  Settings,
  Shield,
  Stethoscope,
  Users,
  WalletCards,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { logout } from "@/features/auth/services/auth-service";
import { cn } from "@/lib/utils";
import { useAuthStore } from "@/store/auth-store";

const navGroups = [
  {
    label: "Inicio",
    items: [{ to: "/", label: "Dashboard", icon: LayoutDashboard }],
  },
  {
    label: "Atención clínica",
    items: [
      { to: "/appointments", label: "Agenda", icon: CalendarDays },
      { to: "/patients", label: "Pacientes", icon: Users },
      { to: "/clinical-records", label: "Expediente", icon: ClipboardPlus },
      { to: "/odontogram", label: "Odontograma", icon: Activity },
      { to: "/periodontal", label: "Periodontal", icon: ChartNoAxesColumn },
      { to: "/files", label: "Archivos", icon: Files },
      { to: "/consents", label: "Consentimientos", icon: ScrollText },
    ],
  },
  {
    label: "Tratamientos y finanzas",
    items: [
      { to: "/treatments", label: "Catálogo", icon: Stethoscope },
      { to: "/treatment-plans", label: "Planes", icon: FileClock },
      { to: "/estimates", label: "Presupuestos", icon: FileText },
      { to: "/payments", label: "Pagos", icon: WalletCards },
      { to: "/cash", label: "Caja", icon: CircleDollarSign },
    ],
  },
  {
    label: "Administración",
    items: [
      { to: "/inventory", label: "Inventario", icon: Package },
      { to: "/suppliers", label: "Proveedores", icon: Building2 },
      { to: "/reports", label: "Reportes", icon: ChartNoAxesColumn },
      { to: "/alerts", label: "Alertas", icon: Bell },
      { to: "/backups", label: "Respaldos", icon: Archive },
    ],
  },
  {
    label: "Sistema",
    items: [
      { to: "/users", label: "Usuarios", icon: Shield },
      { to: "/settings", label: "Configuración", icon: Settings },
    ],
  },
];

export function MainLayout() {
  const [collapsed, setCollapsed] = useState(false);
  const user = useAuthStore((state) => state.user);
  const license = useAuthStore((state) => state.license);
  const sessionToken = useAuthStore((state) => state.sessionToken);
  const clearSession = useAuthStore((state) => state.clearSession);
  const navigate = useNavigate();

  const handleLogout = async () => {
    if (sessionToken) {
      await logout(sessionToken).catch(() => undefined);
    }
    clearSession();
    navigate("/");
  };

  return (
    <TooltipProvider>
      <div className="flex min-h-screen bg-background">
        <aside className={cn("shrink-0 border-r bg-card transition-[width] duration-200", collapsed ? "w-[84px]" : "w-[260px]")}>
          <div className={cn("flex h-16 items-center gap-3 px-4", collapsed && "justify-center px-2")}>
            <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-primary text-primary-foreground">
              <Activity className="h-5 w-5" />
            </div>
            {!collapsed ? (
              <div className="min-w-0">
                <div className="font-semibold leading-none">DentalCare</div>
                <div className="text-xs text-muted-foreground">Manager</div>
              </div>
            ) : null}
          </div>
          <Separator />
          <div className="flex h-12 items-center justify-center border-b px-3">
            <Button
              variant="outline"
              size="sm"
              className={cn("w-full", collapsed && "h-10 w-10 rounded-md border-primary/15 bg-primary/5 px-0 text-primary shadow-none hover:bg-primary/10")}
              onClick={() => setCollapsed((value) => !value)}
            >
              {collapsed ? <ChevronsRight className="h-4 w-4" /> : <><ChevronsLeft className="h-4 w-4" />Contraer</>}
            </Button>
          </div>
          <ScrollArea className="h-[calc(100vh-7rem)]">
            <nav className={cn("grid", collapsed ? "gap-2 px-3 py-3" : "gap-4 p-3")}>
              {navGroups.map((group, index) => (
                <div key={group.label} className="grid gap-1">
                  {!collapsed ? (
                    <div className="px-3 pb-1 text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">{group.label}</div>
                  ) : null}
                  {group.items.map((item) => (
                    <NavEntry key={item.to} item={item} collapsed={collapsed} />
                  ))}
                  {collapsed && index < navGroups.length - 1 ? <div className="mx-auto my-1 h-px w-7 bg-border/80" /> : null}
                </div>
              ))}
            </nav>
          </ScrollArea>
        </aside>

        <div className="flex min-w-0 flex-1 flex-col">
          <header className="flex h-16 items-center justify-between border-b bg-card px-4 sm:px-6">
            <div className="min-w-0">
              <div className="truncate text-sm font-medium">Sistema Integral para Consultorio Dental</div>
              <div className="truncate text-xs text-muted-foreground">Local-first · SQLite · Operación sin internet</div>
            </div>
            <div className="flex items-center gap-3">
              <div className="hidden text-right sm:block">
                <div className="text-sm font-medium">{user?.fullName}</div>
                <div className="text-xs text-muted-foreground">{user?.roleName ?? user?.username}</div>
              </div>
              <Button variant="outline" size="icon" onClick={handleLogout} aria-label="Cerrar sesión">
                <LogOut className="h-4 w-4" />
              </Button>
            </div>
          </header>
          {license && !license.canWrite ? (
            <div className="border-b border-amber-200 bg-amber-50 px-4 py-2 text-sm text-amber-900 sm:px-6">
              <span className="font-medium">Modo sólo lectura.</span> {license.message}
            </div>
          ) : null}
          <main className="min-w-0 flex-1 p-4 sm:p-6">
            <Outlet />
          </main>
        </div>
      </div>
    </TooltipProvider>
  );
}

function NavEntry({
  item,
  collapsed,
}: {
  item: { to: string; label: string; icon: LucideIcon };
  collapsed: boolean;
}) {
  const link = (
    <NavLink
      to={item.to}
      end={item.to === "/"}
      className={({ isActive }) =>
        cn(
          "flex h-10 items-center rounded-md text-sm font-medium text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
          collapsed
            ? "relative mx-auto h-9 w-9 justify-center px-0 text-slate-500 hover:bg-transparent hover:text-primary dark:text-slate-300"
            : "gap-3 px-3",
          isActive
            ? collapsed
              ? "bg-transparent text-primary hover:bg-transparent hover:text-primary"
              : "bg-primary text-primary-foreground shadow-sm hover:bg-primary hover:text-primary-foreground"
            : "",
        )
      }
    >
      <item.icon className={cn("shrink-0", collapsed ? "h-[18px] w-[18px] stroke-[2]" : "h-4 w-4")} />
      {!collapsed ? <span className="truncate">{item.label}</span> : null}
    </NavLink>
  );

  if (!collapsed) {
    return link;
  }

  return (
    <Tooltip>
      <TooltipTrigger asChild>{link}</TooltipTrigger>
      <TooltipContent side="right" sideOffset={10}>{item.label}</TooltipContent>
    </Tooltip>
  );
}
