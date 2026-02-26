"use client";

import { motion } from "framer-motion";

export type FrameworkId = "soc2" | "fedramp" | "gdpr" | "hipaa" | "ccpa";

export interface ComplianceBadgeProps {
  framework: FrameworkId;
  coveragePercent: number;
  sampleLogEntry: string;
  howWePass: string;
  className?: string;
}

const labels: Record<FrameworkId, string> = {
  soc2: "SOC 2 Type II",
  fedramp: "FedRAMP Moderate",
  gdpr: "GDPR",
  hipaa: "HIPAA",
  ccpa: "CCPA",
};

/** Custom SVG checkmark for compliance (no icon libraries per spec) */
function CheckIcon({ className }: { className?: string }) {
  return (
    <svg className={className} width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden>
      <path
        d="M13 4L6 11L3 8"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

/** SOC 2 badge: shield with check (custom SVG, no vendor logo) */
function BadgeSoc2({ className }: { className?: string }) {
  return (
    <svg className={className} width="32" height="32" viewBox="0 0 32 32" fill="none" aria-hidden>
      <path d="M16 4L4 8v6c0 7 6 11 12 14 6-3 12-7 12-14V8L16 4z" stroke="currentColor" strokeWidth="1.5" strokeLinejoin="round" fill="none" />
      <path d="M11 16l3 3 6-6" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

/** GDPR badge: document with lock (custom SVG, no vendor logo) */
function BadgeGdpr({ className }: { className?: string }) {
  return (
    <svg className={className} width="32" height="32" viewBox="0 0 32 32" fill="none" aria-hidden>
      <path d="M8 4h10l6 6v14a2 2 0 01-2 2H8a2 2 0 01-2-2V6a2 2 0 012-2z" stroke="currentColor" strokeWidth="1.5" strokeLinejoin="round" />
      <path d="M18 4v6h6" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      <rect x="11" y="16" width="8" height="6" rx="1" stroke="currentColor" strokeWidth="1.5" />
      <path d="M13 16V14a2 2 0 114 0v2" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
    </svg>
  );
}

export function ComplianceBadge({
  framework,
  coveragePercent,
  sampleLogEntry,
  howWePass,
  className = "",
}: ComplianceBadgeProps) {
  return (
    <motion.article
      initial={{ opacity: 0, y: 12 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ duration: 0.2 }}
      className={`glass rounded-lg p-5 hover:border-[var(--compliance)]/20 transition-colors ${className}`}
    >
      <div className="flex items-center gap-2 mb-3">
        {framework === "soc2" && <BadgeSoc2 className="text-[var(--compliance)] shrink-0" />}
        {framework === "gdpr" && <BadgeGdpr className="text-[var(--compliance)] shrink-0" />}
        {framework !== "soc2" && framework !== "gdpr" && <CheckIcon className="text-[var(--compliance)] shrink-0" />}
        <h3 className="text-lg font-semibold text-[var(--foreground)]">{labels[framework]}</h3>
      </div>
      <p className="text-sm text-[var(--compliance)] font-mono mb-2">
        {coveragePercent}% policy coverage
      </p>
      <pre className="text-xs font-mono text-[var(--muted)] bg-[var(--background)] rounded p-3 overflow-x-auto border border-[var(--border)] mb-3 min-w-0 max-w-full">
        {sampleLogEntry}
      </pre>
      <p className="text-sm text-[var(--muted)]">{howWePass}</p>
    </motion.article>
  );
}
