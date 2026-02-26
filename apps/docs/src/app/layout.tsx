import type { Metadata } from "next";
import Link from "next/link";
import "./globals.css";

export const metadata: Metadata = {
  title: "unblockllm Docs",
  description: "Zero-Trust Privacy Proxy for LLMs — Quickstart, Security Model, SLA.",
};

export default function RootLayout({
  children,
}: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark">
      <body className="min-h-screen bg-slate-950 text-slate-100 antialiased">
        <nav className="border-b border-slate-800 px-6 py-4">
          <div className="max-w-4xl mx-auto flex items-center justify-between">
            <Link href="/" className="font-semibold text-indigo-400">unblockllm Docs</Link>
            <div className="flex gap-6">
              <Link href="/docs/quickstart" className="text-slate-400 hover:text-white">Quickstart</Link>
              <Link href="/docs/security-model" className="text-slate-400 hover:text-white">Security Model</Link>
              <Link href="/docs/sla" className="text-slate-400 hover:text-white">SLA</Link>
            </div>
          </div>
        </nav>
        <main className="max-w-4xl mx-auto px-6 py-10">{children}</main>
      </body>
    </html>
  );
}
