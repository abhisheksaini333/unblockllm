"use client";

import { motion } from "framer-motion";
import { useInView } from "react-intersection-observer";

const DURATION = 0.2;

const tokens = [
  { id: 1, raw: '{"request": "credit_card=XXXX", "action": "BLOCKED"}' },
  { id: 2, raw: '{"request": "email=user@***", "action": "MASKED"}' },
  { id: 3, raw: '{"request": "ssn=***-**-1234", "action": "REDACTED"}' },
  { id: 4, raw: '{"request": "query=weather", "action": "ALLOWED"}' },
];

export function DataFlow() {
  const [ref, inView] = useInView({ triggerOnce: true, threshold: 0.2 });
  const duration = DURATION;

  return (
    <div ref={ref} className="relative rounded-lg border border-[var(--border)] bg-[var(--surface)]/50 p-4 overflow-hidden">
      <div className="flex flex-col sm:flex-row gap-4 sm:gap-6 items-stretch">
        {["Ingest", "Policy", "Redact", "Audit"].map((label, i) => (
          <div key={label} className="flex flex-col items-center flex-1 min-w-0">
            <motion.span
              initial={{ opacity: 0 }}
              animate={inView ? { opacity: 1 } : {}}
              transition={{ duration, delay: i * 0.05 }}
              className="text-xs font-mono uppercase tracking-wider text-[var(--muted)] mb-2"
            >
              {label}
            </motion.span>
            <div className="w-full h-px sm:h-12 sm:w-px bg-[var(--border)] flex-shrink-0" />
          </div>
        ))}
      </div>
      <div className="mt-4 space-y-2">
        {tokens.map((t, i) => (
          <motion.div
            key={t.id}
            initial={{ opacity: 0, x: -20 }}
            animate={inView ? { opacity: 1, x: 0 } : {}}
            transition={{ duration, delay: 0.1 + i * 0.08 }}
            className="font-mono text-xs text-[var(--muted)] truncate terminal-width"
          >
            {t.raw}
          </motion.div>
        ))}
      </div>
    </div>
  );
}
