const USER_ID_KEY = "trading_user_id";
const TOKEN_KEY = "trading_session_token";

export function getStoredUserId(): string | null {
  if (typeof window === "undefined") return null;
  return window.localStorage.getItem(USER_ID_KEY);
}

export function getStoredToken(): string | null {
  if (typeof window === "undefined") return null;
  return window.localStorage.getItem(TOKEN_KEY);
}

export function setStoredSession(userId: string, token: string) {
  window.localStorage.setItem(USER_ID_KEY, userId);
  window.localStorage.setItem(TOKEN_KEY, token);
}

export function clearStoredSession() {
  window.localStorage.removeItem(USER_ID_KEY);
  window.localStorage.removeItem(TOKEN_KEY);
}