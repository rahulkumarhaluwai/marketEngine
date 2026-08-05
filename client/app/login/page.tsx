"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { gqlClient } from "@/lib/graphql-client";
import { LOGIN, REGISTER } from "@/lib/queries";
import { useSession } from "@/app/providers";

type SessionResult = {
  token: string;
  account: { id: string; username: string; cashBalance: string };
};

export default function LoginPage() {
  const router = useRouter();
  const { setSession } = useSession();
  const [mode, setMode] = useState<"login" | "register">("login");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      const mutation = mode === "login" ? LOGIN : REGISTER;
      const data = await gqlClient.request<{ login?: SessionResult; register?: SessionResult }>(mutation, {
        username,
        password,
      });
      const result = data.login ?? data.register;
      if (!result) throw new Error("no session returned");

      setSession(result.account.id, result.token, result.account.username);
      router.push("/");
    } catch (err: any) {
      setError(err?.response?.errors?.[0]?.message ?? "Authentication failed");
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="min-h-screen bg-white dark:bg-gray-950 text-gray-900 dark:text-white p-8 flex justify-center items-center">
      <form onSubmit={handleSubmit} className="max-w-sm w-full flex flex-col gap-4">
        <h1 className="text-2xl font-semibold mb-2">{mode === "login" ? "Log In" : "Register"}</h1>

        <input
          type="text"
          placeholder="Username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          className="bg-gray-100 dark:bg-gray-800 rounded px-3 py-2 border border-gray-300 dark:border-gray-700"
          required
        />
        <input
          type="password"
          placeholder="Password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          className="bg-gray-100 dark:bg-gray-800 rounded px-3 py-2 border border-gray-300 dark:border-gray-700"
          required
        />

        <button type="submit" disabled={loading} className="bg-indigo-600 text-white rounded py-2 font-medium disabled:opacity-50">
          {loading ? "Please wait..." : mode === "login" ? "Log In" : "Register"}
        </button>

        {error && <p className="text-red-500 text-sm">{error}</p>}

        <button
          type="button"
          onClick={() => setMode(mode === "login" ? "register" : "login")}
          className="text-sm text-indigo-600 dark:text-indigo-400 hover:underline"
        >
          {mode === "login" ? "Need an account? Register" : "Already have an account? Log in"}
        </button>
      </form>
    </main>
  );
}