import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { AuthSession, LicenseStatus, UserProfile } from "@/types/shared";

type AuthState = {
  sessionToken: string | null;
  expiresAt: string | null;
  user: UserProfile | null;
  permissions: string[];
  license: LicenseStatus | null;
  locked: boolean;
  setSession: (session: AuthSession) => void;
  clearSession: () => void;
  lock: () => void;
  unlock: () => void;
  hasPermission: (permission: string) => boolean;
};

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      sessionToken: null,
      expiresAt: null,
      user: null,
      permissions: [],
      license: null,
      locked: false,
      setSession: (session) =>
        set({
          sessionToken: session.sessionToken,
          expiresAt: session.expiresAt,
          user: session.user,
          permissions: session.permissions,
          license: session.license,
          locked: false,
        }),
      clearSession: () =>
        set({
          sessionToken: null,
          expiresAt: null,
          user: null,
          permissions: [],
          license: null,
          locked: false,
        }),
      lock: () => set({ locked: true }),
      unlock: () => set({ locked: false }),
      hasPermission: (permission) => get().permissions.includes(permission),
    }),
    {
      name: "dentalcare-auth",
      partialize: (state) => ({
        user: state.user,
        permissions: state.permissions,
        license: state.license,
        locked: state.locked,
      }),
    },
  ),
);
