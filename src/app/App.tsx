import { QueryClient, QueryClientProvider, useQuery } from "@tanstack/react-query";
import { HashRouter, Navigate, Route, Routes } from "react-router-dom";
import { Toaster } from "@/components/ui/sonner";
import { Skeleton } from "@/components/ui/skeleton";
import { SetupPage } from "@/features/auth/pages/SetupPage";
import { LoginPage } from "@/features/auth/pages/LoginPage";
import { getBootstrapStatus } from "@/features/auth/services/auth-service";
import { AppRoutes } from "@/routes/AppRoutes";
import { useInactivityLock } from "@/hooks/use-inactivity-lock";
import { useAuthStore } from "@/store/auth-store";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AppBootstrap />
      <Toaster richColors closeButton />
    </QueryClientProvider>
  );
}

function AppBootstrap() {
  useInactivityLock();
  const sessionToken = useAuthStore((state) => state.sessionToken);
  const locked = useAuthStore((state) => state.locked);
  const { data, isLoading, error } = useQuery({
    queryKey: ["bootstrap-status"],
    queryFn: getBootstrapStatus,
  });

  if (isLoading) {
    return (
      <div className="grid min-h-screen place-items-center bg-background p-6">
        <div className="w-full max-w-md space-y-4">
          <Skeleton className="h-12 w-12 rounded-lg" />
          <Skeleton className="h-8 w-3/4" />
          <Skeleton className="h-40 w-full" />
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="grid min-h-screen place-items-center bg-background p-6">
        <div className="max-w-lg rounded-lg border bg-card p-6 shadow-sm">
          <h1 className="text-lg font-semibold">No se pudo iniciar DentalCare Manager</h1>
          <p className="mt-2 text-sm text-muted-foreground">
            {error instanceof Error ? error.message : String(error)}
          </p>
        </div>
      </div>
    );
  }

  if (data?.requiresSetup) {
    return <SetupPage />;
  }

  if (!sessionToken || locked) {
    return <LoginPage license={data?.license} />;
  }

  return (
    <HashRouter>
      <Routes>
        <Route path="/*" element={<AppRoutes />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </HashRouter>
  );
}
