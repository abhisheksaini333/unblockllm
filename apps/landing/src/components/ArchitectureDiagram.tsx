"use client";

import { useEffect, useRef, useState } from "react";

/* Theme-aligned with landing. Tighter spacing for smaller diagram; classDef ensures text contrast and padding. */
const DIAGRAM = `
%%{init: {'flowchart': {'nodeSpacing': 48, 'rankSpacing': 32}}}%%
flowchart TD
  A[Your App] -->|Unmodified API Call| B[UnblockLLM Proxy]
  B -->|Token Stream Inspection| C{Policy Engine}
  C -->|PII Detected?| D[Redact In-Place]
  C -->|Clean| E[LLM API]
  D -->|Masked Tokens| E
  E -->|Response| F[Audit Log]
  F -->|Immutable| G[SOC 2 Evidence]

  classDef process fill:#1e293b,stroke:#475569,stroke-width:2px,color:#f1f5f9
  classDef decision fill:#1e293b,stroke:#16a34a,stroke-width:2px,color:#f1f5f9
  classDef evidence fill:#1e293b,stroke:#334155,stroke-width:2px,color:#f1f5f9

  class A,B,D,E process
  class C decision
  class F,G evidence

  linkStyle 0,1,2,3,4,5,6 stroke:#64748b,stroke-width:2px
`.trim();

const THEME_VARS = {
  darkMode: true,
  background: "#0f172a",
  primaryColor: "#1e293b",
  primaryTextColor: "#f1f5f9",
  primaryBorderColor: "#475569",
  secondaryColor: "#334155",
  secondaryBorderColor: "#475569",
  tertiaryColor: "#16a34a",
  tertiaryBorderColor: "#15803d",
  lineColor: "#64748b",
  textColor: "#cbd5e1",
  fontFamily: "var(--font-geist-sans), Geist, system-ui, sans-serif",
  fontSize: "14px",
};

export function ArchitectureDiagram({ className = "" }: { className?: string }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    const el = containerRef.current;
    if (!el || typeof window === "undefined") return;

    const id = `mermaid-${Math.random().toString(36).slice(2, 11)}`;

    import("mermaid")
      .then((mermaidModule) => {
        const mermaid = mermaidModule.default;
        mermaid.initialize({
          startOnLoad: false,
          theme: "base",
          securityLevel: "loose",
          themeVariables: THEME_VARS,
          flowchart: {
            useMaxWidth: true,
            htmlLabels: true,
            curve: "basis",
            padding: 10,
          },
        });
        return mermaid.render(id, DIAGRAM);
      })
      .then(({ svg }) => {
        if (el) {
          el.innerHTML = svg;
          const svgEl = el.querySelector("svg");
          if (svgEl) {
            svgEl.setAttribute("role", "img");
            svgEl.setAttribute("aria-label", "UnblockLLM data flow: App to Proxy, Policy Engine, Redact, LLM API, Audit Log, SOC 2 Evidence");
          }
          setReady(true);
        }
      })
      .catch(() => {
        setReady(true);
      });
  }, []);

  return (
    <div
      ref={containerRef}
      className={`min-h-[240px] w-full max-w-xl mx-auto [&_svg]:max-w-full [&_svg]:h-auto [&_svg]:rounded-lg [&_.edgeLabel]:font-sans [&_.edgeLabel]:text-[13px] [&_.edgeLabel]:fill-[var(--foreground-secondary)] [&_.nodeLabel]:font-medium [&_.nodeLabel]:text-[13px] [&_.nodeLabel]:fill-[var(--foreground)] [&_foreignObject]:overflow-visible [&_foreignObject_body]:p-1.5 [&_foreignObject_body]:min-w-0 ${className}`}
      style={{ opacity: ready ? 1 : 0, transition: "opacity 0.3s" }}
    />
  );
}
