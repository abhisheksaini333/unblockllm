"use client";

import { useState, useCallback } from "react";

interface CodeSnippetProps {
  code: string;
  className?: string;
  /** Show terminal prompt ($) before code for CLI-first workflow */
  showPrompt?: boolean;
}

export function CodeSnippet({ code, className = "", showPrompt = false }: CodeSnippetProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [code]);

  return (
    <div className={`rounded-lg overflow-hidden border border-[var(--border-glass)] bg-[var(--surface)]/50 ${className}`}>
      <div className="flex justify-between items-center px-4 py-2 border-b border-[var(--border-glass)] bg-[var(--background)]/80">
        <span className="flex items-center gap-1.5" aria-hidden>
          {showPrompt && (
            <span className="font-mono text-[var(--compliance)] text-sm mr-1" aria-hidden>$</span>
          )}
          <span className="w-2.5 h-2.5 rounded-full bg-[var(--critical)]" />
          <span className="w-2.5 h-2.5 rounded-full bg-amber-500" />
          <span className="w-2.5 h-2.5 rounded-full bg-[var(--compliance)]" />
        </span>
        <button
          type="button"
          onClick={handleCopy}
          className="text-sm text-[var(--foreground-secondary)] hover:text-[var(--compliance)] transition-colors cursor-pointer"
        >
          {copied ? "Copied!" : "Copy"}
        </button>
      </div>
      <pre className="p-4 font-mono text-sm text-[var(--foreground-secondary)] overflow-x-auto min-w-0 max-w-full">
        <code>{code}</code>
      </pre>
    </div>
  );
}
