import { invokeCommand } from "@/lib/api";
import type { AuthSession, BootstrapStatus, UserListItem } from "@/types/shared";
import type { LoginFormValues, SetupFormValues } from "@/features/auth/schemas/auth-schemas";

export function getBootstrapStatus() {
  return invokeCommand<BootstrapStatus>("get_bootstrap_status");
}

export function setupClinicAndAdmin(input: SetupFormValues) {
  return invokeCommand<AuthSession>("setup_clinic_and_admin", { input });
}

export function login(input: LoginFormValues) {
  return invokeCommand<AuthSession>("login", { input });
}

export function logout(sessionToken: string) {
  return invokeCommand<void>("logout", { sessionToken });
}

export function listUsers(sessionToken: string) {
  return invokeCommand<UserListItem[]>("list_users", { sessionToken });
}
