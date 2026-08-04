"use client";

import { createContext, useContext, useEffect, useState, ReactNode } from "react";
import { getStoredUserId, getStoredToken, setStoredSession, clearStoredSession } from "@/lib/session";
import { gqlClient } from "@/lib/graphql-client";
import { LOGOUT } from "@/lib/queries";
import { Theme, getStoredTheme, setStoredTheme, applyTheme } from "@/lib/theme";

type SessionContextValue = {
  userId: string | null;
  token: string | null;
  setSession: (userId: string, token: string) => void;
  clearUserId: () => void;
};

type ThemeContextValue = {
  theme: Theme;
  toggleTheme: () => void;
};

const SessionContext = createContext<SessionContextValue | undefined>(undefined);
const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);

export function Providers({ children }: { children: ReactNode }) {
  const [userId, setUserIdState] = useState<string | null>(null);
  const [token, setTokenState] = useState<string | null>(null);
  const [theme, setThemeState] = useState<Theme>("dark");

  useEffect(() => {
    setUserIdState(getStoredUserId());
    setTokenState(getStoredToken());

    const initialTheme = getStoredTheme();
    setThemeState(initialTheme);
    applyTheme(initialTheme);
  }, []);

  const setSession = (userId: string, token: string) => {
    setStoredSession(userId, token);
    setUserIdState(userId);
    setTokenState(token);
  };

  const clearUserId = () => {
    if (token) {
      gqlClient.request(LOGOUT, { token }).catch(() => {});
    }
    clearStoredSession();
    setUserIdState(null);
    setTokenState(null);
  };

  const toggleTheme = () => {
    const next: Theme = theme === "dark" ? "light" : "dark";
    setThemeState(next);
    setStoredTheme(next);
    applyTheme(next);
  };

  return (
    <SessionContext.Provider value={{ userId, token, setSession, clearUserId }}>
      <ThemeContext.Provider value={{ theme, toggleTheme }}>{children}</ThemeContext.Provider>
    </SessionContext.Provider>
  );
}

export function useSession() {
  const ctx = useContext(SessionContext);
  if (!ctx) throw new Error("useSession must be used within Providers");
  return ctx;
}

export function useTheme() {
  const ctx = useContext(ThemeContext);
  if (!ctx) throw new Error("useTheme must be used within Providers");
  return ctx;
}