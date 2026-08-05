const USER_ID_KEY = "trading_user_id";
const TOKEN_KEY = "trading_session_token";
const USERNAME_KEY = "trading_username";

export function getStoredUserId(): string | null {
  if (typeof window === "undefined") return null;
  return window.localStorage.getItem(USER_ID_KEY);
}

export function getStoredToken(): string | null {
  if (typeof window === "undefined") return null;
  return window.localStorage.getItem(TOKEN_KEY);
}

export function getStoredUsername(): string | null {
  if (typeof window === "undefined") return null;
  return window.localStorage.getItem(USERNAME_KEY);
}

export function setStoredSession(userId: string, token: string, username: string) {
  window.localStorage.setItem(USER_ID_KEY, userId);
  window.localStorage.setItem(TOKEN_KEY, token);
  window.localStorage.setItem(USERNAME_KEY, username);
}

export function clearStoredSession() {
  window.localStorage.removeItem(USER_ID_KEY);
  window.localStorage.removeItem(TOKEN_KEY);
  window.localStorage.removeItem(USERNAME_KEY);
}