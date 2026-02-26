import Link from "next/link";

const GITHUB_DOCS_BLOB = "https://github.com/unblockllm/unblockllm/blob/main/docs";

const DOC_LINKS = [
  { title: "Architecture", href: `${GITHUB_DOCS_BLOB}/ARCHITECTURE.md`, description: "System architecture, data flow, and Mermaid diagrams." },
  { title: "Deployment", href: `${GITHUB_DOCS_BLOB}/DEPLOYMENT.md`, description: "Production deployment, TLS, secrets, Redis, and audit retention." },
  { title: "Local development", href: `${GITHUB_DOCS_BLOB}/LOCAL_DEVELOPMENT.md`, description: "Run proxy and dashboard locally." },
  { title: "Benchmark reports", href: `${GITHUB_DOCS_BLOB}/MODEL_BENCHMARK_REPORT.md`, description: "Model benchmark and performance report." },
];

export const metadata = {
  title: "Docs | UnblockLLM",
  description: "UnblockLLM documentation — deployment, local development, and architecture.",
};

export default function DocsPage() {
  return (
    <div className="min-h-screen w-full">
      <nav className="border-b border-[var(--border-glass)] bg-[var(--background)]/95 backdrop-blur-[12px]">
        <div className="w-full max-w-6xl mx-auto px-4 sm:px-6 py-3 flex items-center justify-between">
          <Link href="/" className="flex items-center gap-2 text-lg font-semibold font-mono text-[var(--foreground)]">
            <img src="/logo-mark.svg" alt="" width="24" height="24" className="shrink-0" />
            UnblockLLM
          </Link>
          <div className="flex items-center gap-4">
            <Link href="/" className="text-sm text-[var(--foreground-secondary)] hover:text-[var(--foreground)] transition-colors">
              Home
            </Link>
            <Link href="/#how-it-works" className="text-sm text-[var(--foreground-secondary)] hover:text-[var(--foreground)] transition-colors">
              How it works
            </Link>
            <Link href="/#contact" className="text-sm text-[var(--foreground-secondary)] hover:text-[var(--foreground)] transition-colors">
              Contact
            </Link>
          </div>
        </div>
      </nav>

      <main className="w-full max-w-6xl mx-auto px-4 sm:px-6 py-16">
        <h1 className="text-2xl sm:text-3xl font-bold text-[var(--foreground)] mb-2">Documentation</h1>
        <p className="text-[var(--foreground-secondary)] mb-10 max-w-xl">
          Deployment guides, architecture overview, and local development for the UnblockLLM zero-trust proxy.
        </p>

        <ul className="space-y-4">
          {DOC_LINKS.map((doc) => (
            <li key={doc.href}>
              <a
                href={doc.href}
                target={doc.href.startsWith("http") ? "_blank" : undefined}
                rel={doc.href.startsWith("http") ? "noopener noreferrer" : undefined}
                className="block rounded-lg border border-[var(--border-glass)] bg-[var(--surface)]/40 p-4 hover:bg-[var(--surface)]/70 transition-colors"
              >
                <span className="font-medium text-[var(--foreground)]">{doc.title}</span>
                <p className="mt-1 text-sm text-[var(--foreground-secondary)]">{doc.description}</p>
              </a>
            </li>
          ))}
        </ul>

        <div className="mt-12 p-4 rounded-lg border border-[var(--border-glass)] bg-[var(--surface)]/30">
          <p className="text-sm text-[var(--foreground-secondary)]">
            <strong className="text-[var(--foreground)]">Contact:</strong>{" "}
            <a href="mailto:hello@unblockllm.dev" className="text-[var(--compliance)] hover:underline">
              hello@unblockllm.dev
            </a>
          </p>
        </div>
      </main>
    </div>
  );
}
