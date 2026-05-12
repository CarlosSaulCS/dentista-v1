import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { AuthSession, UserProfile } from "@/types/shared";

type AuthState = {
  sessionToken: string | null;
  expiresAt: string | null;
  user: UserProfile | null;
  permissions: string[];
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
      locked: false,
      setSession: (session) =>
        set({
          sessionToken: session.sessionToken,
          expiresAt: session.expiresAt,
          user: session.user,
          permissions: session.permissions,
          locked: false,
        }),
      clearSession: () =>
        set({
          sessionToken: null,
          expiresAt: null,
          user: null,
          permissions: [],
          locked: false,
        }),
      lock: () => set({ locked: true }),
      unlock: () => set({ locked: false }),
      hasPermission: (permission) => get().permissions.includes(permission),
    }),
    {
      name: "dentalcare-auth",
      partialize: (state) => ({
        sessionToken: state.sessionToken,
        expiresAt: state.expiresAt,
        user: state.user,
        permissions: state.permissions,
        locked: state.locked,
      }),
    },
  ),
);
