import { useEffect } from "react";
import { useAuthStore } from "@/store/auth-store";

const INACTIVITY_MS = 15 * 60 * 1000;

export function useInactivityLock() {
  const sessionToken = useAuthStore((state) => state.sessionToken);
  const lock = useAuthStore((state) => state.lock);

  useEffect(() => {
    if (!sessionToken) return;

    let timeout = window.setTimeout(lock, INACTIVITY_MS);
    const reset = () => {
      window.clearTimeout(timeout);
      timeout = window.setTimeout(lock, INACTIVITY_MS);
    };

    window.addEventListener("mousemove", reset);
    window.addEventListener("keydown", reset);
    window.addEventListener("click", reset);

    return () => {
      window.clearTimeout(timeout);
      window.removeEventListener("mousemove", reset);
      window.removeEventListener("keydown", reset);
      window.removeEventListener("click", reset);
    };
  }, [lock, sessionToken]);
}
