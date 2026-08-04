"use client";

import { createContext, useContext, useEffect, useState, ReactNode } from "react";
import { getStoredUserId, getStoredToken, setStoredSession, clearStoredSession } from "@/lib/session";
import { gqlClient } from "@/lib/graphql-client";
import { LOGOUT } from "@/lib/queries";

type SessionContextValue = {
  userId: string | null;
  token: string | null;
  setSession: (userId: string, token: string) => void;
  clearUserId: () => void;
};

const SessionContext = createContext<SessionContextValue | undefined>(undefined);

export function Providers({ children }: { children: ReactNode }) {
  const [userId, setUserIdState] = useState<string | null>(null);
  const [token, setTokenState] = useState<string | null>(null);

  useEffect(() => {
    setUserIdState(getStoredUserId());
    setTokenState(getStoredToken());
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

  return (
    <SessionContext.Provider value={{ userId, token, setSession, clearUserId }}>
      {children}
    </SessionContext.Provider>
  );
}

export function useSession() {
  const ctx = useContext(SessionContext);
  if (!ctx) throw new Error("useSession must be used within Providers");
  return ctx;
}