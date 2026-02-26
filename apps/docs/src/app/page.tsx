import Link from "next/link";

export default function DocsHome() {
  return (
    <div>
      <h1 className="text-3xl font-bold mb-2">Documentation</h1>
      <p className="text-slate-400 mb-10">
        Zero-Trust Privacy Proxy for LLMs. Redact PII locally, stream to any model.
      </p>
      <ul className="space-y-4">
        <li>
          <Link href="/docs/quickstart" className="text-indigo-400 hover:underline font-medium">
            Quickstart
          </Link>
          <p className="text-slate-500 text-sm mt-1">Get the proxy running and send your first request.</p>
        </li>
        <li>
          <Link href="/docs/security-model" className="text-indigo-400 hover:underline font-medium">
            Security Model
          </Link>
          <p className="text-slate-500 text-sm mt-1">Zero-trust overview, audit schema, sub-processors.</p>
        </li>
        <li>
          <Link href="/docs/sla" className="text-indigo-400 hover:underline font-medium">
            SLA
          </Link>
          <p className="text-slate-500 text-sm mt-1">Overhead targets: Regex &lt;5ms, ONNX &lt;50ms.</p>
        </li>
      </ul>
    </div>
  );
}
