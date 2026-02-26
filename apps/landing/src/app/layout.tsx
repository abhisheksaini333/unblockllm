import type { Metadata, Viewport } from "next";
import { GeistSans } from "geist/font/sans";
import { GeistMono } from "geist/font/mono";
import "./globals.css";

const baseUrl = process.env.NEXT_PUBLIC_BASE_URL ?? "https://unblockllm.dev";

export const metadata: Metadata = {
  title: "UnblockLLM | Zero-Trust Privacy Proxy for LLMs",
  description:
    "Secure your LLM traffic with zero-trust proxy. Block PII, enforce policies, and audit every token without sacrificing performance.",
  openGraph: {
    title: "UnblockLLM | Zero-Trust Privacy Proxy for LLMs",
    description:
      "Zero-trust proxy for OpenAI, Anthropic, and custom LLMs — block PII, enforce policies, audit every token.",
    url: baseUrl,
    siteName: "UnblockLLM",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "UnblockLLM | Zero-Trust Privacy Proxy for LLMs",
  },
  metadataBase: new URL(baseUrl),
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  themeColor: "#0f172a",
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en" className={`dark ${GeistSans.variable} ${GeistMono.variable}`} suppressHydrationWarning>
      <body className="font-sans antialiased min-h-screen bg-[var(--background)] text-[var(--foreground)]" suppressHydrationWarning>
        {children}
      </body>
    </html>
  );
}
