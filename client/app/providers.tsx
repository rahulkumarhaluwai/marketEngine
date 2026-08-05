"use client";

import { createContext, useContext, useEffect, useState, ReactNode } from "react";
import {
  getStoredUserId,
  getStoredToken,
  getStoredUsername,
  setStoredSession,
  clearStoredSession,
} from "@/lib/session";
import { gqlClient } from "@/lib/graphql-client";
import { LOGOUT } from "@/lib/queries";
import { Theme, getStoredTheme, setStoredTheme, applyTheme } from "@/lib/theme";

type SessionContextValue = {
  userId: string | null;
  token: string | null;
  username: string | null;
  setSession: (userId: string, token: string, username: string) => void;
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
  const [username, setUsernameState] = useState<string | null>(null);
  const [theme, setThemeState] = useState<Theme>("dark");

  useEffect(() => {
    setUserIdState(getStoredUserId());
    setTokenState(getStoredToken());
    setUsernameState(getStoredUsername());

    const initialTheme = getStoredTheme();
    setThemeState(initialTheme);
    applyTheme(initialTheme);
  }, []);

  const setSession = (userId: string, token: string, username: string) => {
    setStoredSession(userId, token, username);
    setUserIdState(userId);
    setTokenState(token);
    setUsernameState(username);
  };

  const clearUserId = () => {
    if (token) {
      gqlClient.request(LOGOUT, { token }).catch(() => {});
    }
    clearStoredSession();
    setUserIdState(null);
    setTokenState(null);
    setUsernameState(null);
  };

  const toggleTheme = () => {
    const next: Theme = theme === "dark" ? "light" : "dark";
    setThemeState(next);
    setStoredTheme(next);
    applyTheme(next);
  };

  return (
    <SessionContext.Provider value={{ userId, token, username, setSession, clearUserId }}>
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