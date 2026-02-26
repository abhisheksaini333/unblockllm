"use client";

import { useState, useEffect, useCallback } from "react";
import { motion } from "framer-motion";
import { IconTerminal } from "./icons";

/** Typewriter ~65 wpm (~5.4 c/s) = ~185ms per char. Copy button + Run in Terminal simulation. */
const CHAR_DELAY_MS = 185;

interface TerminalDemoProps {
  command: string;
  response?: string;
  onComplete?: () => void;
  className?: string;
  showCopy?: boolean;
}

export function TerminalDemo({
  command,
  response,
  onComplete,
  className = "",
  showCopy = true,
}: TerminalDemoProps) {
  const [typed, setTyped] = useState("");
  const [showResponse, setShowResponse] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (typed.length >= command.length) {
      if (response) {
        const t = setTimeout(() => setShowResponse(true), 300);
        return () => clearTimeout(t);
      }
      onComplete?.();
      return;
    }
    const t = setTimeout(() => setTyped(command.slice(0, typed.length + 1)), CHAR_DELAY_MS);
    return () => clearTimeout(t);
  }, [command, typed, response, onComplete]);

  const handleCopy = useCallback(async () => {
    await navigator.clipboard.writeText(command);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [command]);

  return (
    <div className={`rounded-lg border border-[var(--border-glass)] bg-[var(--background)] overflow-hidden ${className}`}>
      <div className="flex items-center justify-between px-3 py-2 border-b border-[var(--border-glass)]">
        <span className="flex items-center gap-2 text-xs font-mono text-[var(--muted)]">
          <IconTerminal className="w-4 h-4" />
          Terminal
        </span>
        {showCopy && (
          <button
            type="button"
            onClick={handleCopy}
            className="text-xs font-mono text-[var(--muted)] hover:text-[var(--foreground)] transition-colors cursor-pointer"
          >
            {copied ? "Copied" : "Copy"}
          </button>
        )}
      </div>
      <div className="p-4 font-mono text-sm text-[var(--foreground-secondary)] terminal-width">
        <div className="flex items-baseline gap-1">
          <span className="text-[var(--compliance)]">$</span>
          <span>{typed}</span>
          <span className="terminal-cursor" />
        </div>
        {showResponse && response && (
          <motion.pre
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.2 }}
            className="mt-3 text-xs text-[var(--muted)] whitespace-pre-wrap"
          >
            {response}
          </motion.pre>
        )}
      </div>
    </div>
  );
}
