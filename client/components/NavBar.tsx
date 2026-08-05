"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useSession } from "@/app/providers";
import { ThemeToggle } from "./ThemeToggle";
import { Avatar } from "./Avatar";

const LINKS = [
  { href: "/", label: "Dashboard" },
  { href: "/trade", label: "Trade" },
  { href: "/portfolio", label: "Portfolio" },
  { href: "/history", label: "History" },
  { href: "/alerts", label: "Alerts" },
  { href: "/deposit", label: "Deposit" },
  { href: "/leaderboard", label: "Leaderboard" },
];

export function NavBar() {
  const pathname = usePathname();
  const { userId, username, clearUserId } = useSession();

  return (
    <header className="border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950">
      <div className="max-w-6xl mx-auto px-8 py-4 flex items-center justify-between">
        <div className="flex items-center gap-8">
          <nav className="flex gap-6">
            {LINKS.map((link) => {
              const active = pathname === link.href;
              return (
                <Link
                  key={link.href}
                  href={link.href}
                  className={`text-sm ${
                    active
                      ? "text-gray-900 dark:text-white font-medium"
                      : "text-gray-500 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200"
                  }`}
                >
                  {link.label}
                </Link>
              );
            })}
          </nav>
        </div>

        <div className="flex items-center gap-4">
          <ThemeToggle />
          {userId && username ? (
            <>
              <Avatar username={username} size="sm" />
              <span className="text-sm text-gray-600 dark:text-gray-300">{username}</span>
              <button onClick={clearUserId} className="text-sm text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white">
                Log out
              </button>
            </>
          ) : (
            <Link href="/login" className="text-sm text-indigo-600 dark:text-indigo-400 hover:underline">
              Log in
            </Link>
          )}
        </div>
      </div>
    </header>
  );
}