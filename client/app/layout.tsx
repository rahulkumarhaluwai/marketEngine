import type { Metadata } from "next";
import { Providers } from "./providers";
import { NavBar } from "@/components/NavBar";
import "./globals.css";

export const metadata: Metadata = {
  title: "Market Engine",
  description: "Simulated real-time trading platform",
};

const THEME_INIT_SCRIPT = `
(function() {
  try {
    var theme = localStorage.getItem('trading_theme') || 'dark';
    if (theme === 'dark') document.documentElement.classList.add('dark');
  } catch (e) {}
})();
`;

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: THEME_INIT_SCRIPT }} />
      </head>
      <body className="bg-white dark:bg-gray-950 transition-colors" suppressHydrationWarning>
        <Providers>
          <NavBar />
          {children}
        </Providers>
      </body>
    </html>
  );
}