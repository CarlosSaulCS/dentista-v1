import { lazy, Suspense, type ReactNode } from "react";
import { Navigate, Route, Routes } from "react-router-dom";
import { MainLayout } from "@/layouts/MainLayout";
import { Skeleton } from "@/components/ui/skeleton";

const DashboardPage = lazy(() => import("@/features/dashboard/pages/DashboardPage").then((module) => ({ default: module.DashboardPage })));
const PatientsPage = lazy(() => import("@/features/patients/pages/PatientsPage").then((module) => ({ default: module.PatientsPage })));
const AppointmentsPage = lazy(() => import("@/features/appointments/pages/AppointmentsPage").then((module) => ({ default: module.AppointmentsPage })));
const ClinicalRecordsPage = lazy(() => import("@/features/clinical-records/pages/ClinicalRecordsPage").then((module) => ({ default: module.ClinicalRecordsPage })));
const OdontogramPage = lazy(() => import("@/features/odontogram/pages/OdontogramPage").then((module) => ({ default: module.OdontogramPage })));
const BackupsPage = lazy(() => import("@/features/backups/pages/BackupsPage").then((module) => ({ default: module.BackupsPage })));
const UsersPage = lazy(() => import("@/features/users/pages/UsersPage").then((module) => ({ default: module.UsersPage })));
const TreatmentsPage = lazy(() => import("@/features/treatments/pages/TreatmentsPage").then((module) => ({ default: module.TreatmentsPage })));
const TreatmentPlansPage = lazy(() => import("@/features/treatment-plans/pages/TreatmentPlansPage").then((module) => ({ default: module.TreatmentPlansPage })));
const EstimatesPage = lazy(() => import("@/features/estimates/pages/EstimatesPage").then((module) => ({ default: module.EstimatesPage })));
const PaymentsPage = lazy(() => import("@/features/payments/pages/PaymentsPage").then((module) => ({ default: module.PaymentsPage })));
const CashPage = lazy(() => import("@/features/cash/pages/CashPage").then((module) => ({ default: module.CashPage })));
const FilesPage = lazy(() => import("@/features/files/pages/FilesPage").then((module) => ({ default: module.FilesPage })));
const ConsentsPage = lazy(() => import("@/features/consents/pages/ConsentsPage").then((module) => ({ default: module.ConsentsPage })));
const InventoryPage = lazy(() => import("@/features/inventory/pages/InventoryPage").then((module) => ({ default: module.InventoryPage })));
const SuppliersPage = lazy(() => import("@/features/suppliers/pages/SuppliersPage").then((module) => ({ default: module.SuppliersPage })));
const ReportsPage = lazy(() => import("@/features/reports/pages/ReportsPage").then((module) => ({ default: module.ReportsPage })));
const AlertsPage = lazy(() => import("@/features/alerts/pages/AlertsPage").then((module) => ({ default: module.AlertsPage })));
const PeriodontalPage = lazy(() => import("@/features/periodontal/pages/PeriodontalPage").then((module) => ({ default: module.PeriodontalPage })));
const SettingsPage = lazy(() => import("@/features/settings/pages/SettingsPage").then((module) => ({ default: module.SettingsPage })));

function route(element: ReactNode) {
  return <Suspense fallback={<RouteFallback />}>{element}</Suspense>;
}

function RouteFallback() {
  return (
    <div className="space-y-6">
      <div className="space-y-2">
        <Skeleton className="h-8 w-64" />
        <Skeleton className="h-4 w-96 max-w-full" />
      </div>
      <div className="grid gap-4 md:grid-cols-3">
        <Skeleton className="h-28" />
        <Skeleton className="h-28" />
        <Skeleton className="h-28" />
      </div>
    </div>
  );
}

export function AppRoutes() {
  return (
    <Routes>
      <Route element={<MainLayout />}>
        <Route index element={route(<DashboardPage />)} />
        <Route path="patients" element={route(<PatientsPage />)} />
        <Route path="appointments" element={route(<AppointmentsPage />)} />
        <Route path="clinical-records" element={route(<ClinicalRecordsPage />)} />
        <Route path="odontogram" element={route(<OdontogramPage />)} />
        <Route path="backups" element={route(<BackupsPage />)} />
        <Route path="users" element={route(<UsersPage />)} />
        <Route path="treatments" element={route(<TreatmentsPage />)} />
        <Route path="treatment-plans" element={route(<TreatmentPlansPage />)} />
        <Route path="estimates" element={route(<EstimatesPage />)} />
        <Route path="payments" element={route(<PaymentsPage />)} />
        <Route path="cash" element={route(<CashPage />)} />
        <Route path="files" element={route(<FilesPage />)} />
        <Route path="consents" element={route(<ConsentsPage />)} />
        <Route path="inventory" element={route(<InventoryPage />)} />
        <Route path="suppliers" element={route(<SuppliersPage />)} />
        <Route path="reports" element={route(<ReportsPage />)} />
        <Route path="alerts" element={route(<AlertsPage />)} />
        <Route path="periodontal" element={route(<PeriodontalPage />)} />
        <Route path="settings" element={route(<SettingsPage />)} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  );
}
