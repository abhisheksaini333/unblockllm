"use client";

import { useState, useCallback } from "react";
import { motion } from "framer-motion";
import Link from "next/link";
import { DataFlow } from "@/components/DataFlow";
import { ComplianceBadge } from "@/components/ComplianceBadge";
import { LiveMetric } from "@/components/LiveMetric";
import { ArchitectureDiagram } from "@/components/ArchitectureDiagram";
import { StepExplanation, HOW_IT_WORKS_STEPS } from "@/components/StepExplanation";
import { CodeSnippet } from "@/components/CodeSnippet";
import { SkillsIntegration } from "@/components/SkillsIntegration";
import { PolicyFlowAnimation } from "@/components/PolicyFlowAnimation";
import { IconGitHub } from "@/components/icons";

const MOTION = { duration: 0.2 };
const CONTACT_EMAIL = "hello@unblockllm.dev";

function EmailUsButton({ email }: { email: string }) {
  const handleClick = useCallback(() => {
    navigator.clipboard.writeText(email).catch(() => {});
    window.location.href = `mailto:${email}`;
  }, [email]);
  return (
    <button
      type="button"
      onClick={handleClick}
      className="inline-flex items-center gap-2 rounded-lg bg-[var(--compliance)] px-4 py-2.5 text-sm font-medium text-[var(--background)] hover:opacity-90 transition-opacity cursor-pointer min-h-[44px]"
      aria-label="Open email client to contact us"
    >
      Email us
    </button>
  );
}

function CopyEmailButton({ email }: { email: string }) {
  const [copied, setCopied] = useState(false);
  const copy = useCallback(() => {
    navigator.clipboard.writeText(email).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }, [email]);
  return (
    <button
      type="button"
      onClick={copy}
      className="inline-flex items-center gap-2 rounded-lg border border-[var(--border-glass)] bg-transparent px-4 py-2.5 text-sm font-medium text-[var(--foreground)] hover:bg-[var(--surface)] transition-colors cursor-pointer min-h-[44px]"
      aria-label="Copy email address"
    >
      {copied ? "Copied!" : "Copy email"}
    </button>
  );
}

function TrustBadge({ label }: { label: string }) {
  return (
    <span className="inline-flex items-center gap-1.5 rounded border border-[var(--border-glass)] bg-[var(--surface)]/60 px-2.5 py-1 text-xs font-mono text-[var(--foreground-secondary)]">
      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden>
        <path d="M8 3L4 7L2 5" stroke="var(--compliance)" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
      {label}
    </span>
  );
}

export default function LandingPage() {
  const [activeStep, setActiveStep] = useState(0);
  const currentCode = HOW_IT_WORKS_STEPS[activeStep]?.codeSnippet ?? HOW_IT_WORKS_STEPS[0].codeSnippet;

  return (
    <div className="min-h-screen w-full overflow-x-hidden">
      {/* Nav */}
      <nav className="fixed top-0 left-0 right-0 z-50 border-b border-[var(--border-glass)] bg-[var(--background)]/95 backdrop-blur-[12px]">
        <div className="w-full max-w-6xl mx-auto px-4 sm:px-6 py-3 flex flex-wrap items-center justify-between gap-4">
          <Link href="/" className="flex items-center gap-2 text-lg font-semibold font-mono text-[var(--foreground)]">
            <img src="/logo-mark.svg" alt="" width="24" height="24" className="shrink-0" />
            UnblockLLM
          </Link>
          <div className="flex items-center gap-4 flex-wrap">
            <Link href="#how-it-works" className="text-sm text-[var(--foreground-secondary)] hover:text-[var(--foreground)] transition-colors">How it works</Link>
            <Link href="#compliance" className="text-sm text-[var(--foreground-secondary)] hover:text-[var(--foreground)] transition-colors">Compliance</Link>
            <Link href="#skills" className="text-sm text-[var(--foreground-secondary)] hover:text-[var(--foreground)] transition-colors">Skills.sh</Link>
            <Link href="#enterprise" className="text-sm text-[var(--foreground-secondary)] hover:text-[var(--foreground)] transition-colors">Enterprise</Link>
            <a href="https://github.com/unblockllm/unblockllm" target="_blank" rel="noopener noreferrer" className="text-sm text-[var(--foreground-secondary)] hover:text-[var(--foreground)] transition-colors inline-flex items-center gap-1">
              <IconGitHub className="w-4 h-4" /> GitHub
            </a>
            <div className="flex items-center gap-2">
              <TrustBadge label="SOC 2 Type II" />
              <TrustBadge label="FedRAMP Moderate" />
              <TrustBadge label="GDPR" />
            </div>
          </div>
        </div>
      </nav>

      {/* Hero — 90vh max, right-aligned text, CTA above fold */}
      <header className="relative min-h-[90vh] w-full flex flex-col lg:flex-row items-center justify-end lg:justify-between gap-12 pt-28 pb-16 px-4 sm:px-6 max-w-6xl mx-auto">
        <PolicyFlowAnimation policyEngineHighlight={activeStep === 1} />
        <div className="w-full lg:max-w-[50%] order-2 lg:order-1 relative z-10 min-w-0">
          <DataFlow />
        </div>
        <div className="w-full lg:max-w-[50%] text-left lg:text-right order-1 lg:order-2 relative z-10 min-w-0">
          <motion.h1 initial={{ opacity: 0, y: 12 }} animate={{ opacity: 1, y: 0 }} transition={MOTION} className="text-3xl sm:text-4xl lg:text-5xl font-bold tracking-tight text-[var(--foreground)]">
            LLM Privacy Without Sacrificing Performance
          </motion.h1>
          <motion.p
            initial={{ opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ ...MOTION, delay: 0.05 }}
            className="mt-4 text-base sm:text-lg text-[var(--foreground-secondary)] max-w-xl lg:ml-auto"
          >
            Zero-trust proxy for OpenAI, Anthropic, and custom LLMs — block PII, enforce policies, and audit <em>every</em> token.
          </motion.p>
          <motion.div initial={{ opacity: 0, y: 12 }} animate={{ opacity: 1, y: 0 }} transition={{ ...MOTION, delay: 0.1 }} className="mt-6 flex flex-col sm:flex-row gap-4 lg:justify-end">
            <LiveMetric value={97.3} prefix="↓ " suffix="%" label="PII exposure" duration={1.2} pulseOnComplete />
          </motion.div>
          <motion.div initial={{ opacity: 0, y: 12 }} animate={{ opacity: 1, y: 0 }} transition={{ ...MOTION, delay: 0.15 }} className="mt-8">
            <a href="https://github.com/unblockllm/unblockllm/blob/main/docs/ARCHITECTURE.md" target="_blank" rel="noopener noreferrer" className="inline-flex items-center gap-2 rounded-lg bg-[var(--compliance)] px-5 py-2.5 text-sm font-medium text-[var(--background)] hover:opacity-90 transition-opacity cursor-pointer">
              See Architecture →
            </a>
          </motion.div>
        </div>
      </header>

      {/* How It Works — 12-col grid: 4 diagram | 4 steps | 4 code. Vertical on mobile. */}
      <section id="how-it-works" className="py-20 px-4 sm:px-6 border-t border-[var(--border-glass)] w-full">
        <div className="w-full max-w-6xl mx-auto min-w-0">
          <h2 className="text-2xl font-bold text-[var(--foreground)] mb-8">How it works</h2>
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 lg:gap-8">
            <div className="lg:col-span-4 lg:sticky lg:top-24 lg:self-start min-w-0">
              <ArchitectureDiagram className="w-full" />
            </div>
            <div className="lg:col-span-4 min-w-0">
              <StepExplanation activeStep={activeStep} onStepChange={setActiveStep} />
            </div>
            <div className="lg:col-span-4 lg:sticky lg:top-24 lg:self-start min-w-0">
              <CodeSnippet code={currentCode} showPrompt />
            </div>
          </div>
        </div>
      </section>

      {/* Compliance Hub */}
      <section id="compliance" className="py-20 px-4 sm:px-6 border-t border-[var(--border-glass)] w-full">
        <div className="w-full max-w-6xl mx-auto min-w-0">
          <h2 className="text-2xl font-bold text-[var(--foreground)] mb-8">Compliance Hub</h2>
          <div className="flex flex-wrap items-center gap-3 mb-8">
            <div className="flex items-center gap-2 rounded-full border border-[var(--border-glass)] bg-[var(--surface)]/60 px-3 py-1.5">
              <span className="font-mono text-[var(--compliance)] font-semibold">97.3%</span>
              <span className="text-sm text-[var(--foreground-secondary)]">PII exposure reduction</span>
            </div>
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
            <ComplianceBadge framework="soc2" coveragePercent={99} sampleLogEntry='{"audit":"token_audit","scope":"egress","ts":"2025-01-15T12:00:00Z"}' howWePass="UnblockLLM supports SOC 2 Type II with immutable audit logs and no PII in evidence." />
            <ComplianceBadge framework="hipaa" coveragePercent={98} sampleLogEntry='{"entity":"PHI","action":"redacted","ts":"2025-01-15T12:00:00Z"}' howWePass="UnblockLLM passes HIPAA by never logging PHI and enforcing redaction before egress." />
            <ComplianceBadge framework="gdpr" coveragePercent={100} sampleLogEntry='{"request_id":"req_xyz","entity_types":["EMAIL"],"region":"EU"}' howWePass="UnblockLLM passes GDPR via data minimization and configurable retention (90-day default)." />
            <ComplianceBadge framework="ccpa" coveragePercent={97} sampleLogEntry='{"consumer_data":"masked","jurisdiction":"CA"}' howWePass="UnblockLLM passes CCPA by masking PII in real time and supporting right-to-deletion workflows." />
          </div>
        </div>
      </section>

      {/* Skills.sh — SkillsIntegration component */}
      <section id="skills" className="py-20 px-4 sm:px-6 border-t border-[var(--border-glass)] w-full">
        <div className="w-full max-w-6xl mx-auto min-w-0">
          <SkillsIntegration />
        </div>
      </section>

      {/* Enterprise Proof */}
      <section id="enterprise" className="py-20 px-4 sm:px-6 border-t border-[var(--border-glass)] w-full">
        <div className="w-full max-w-6xl mx-auto min-w-0">
          <h2 className="text-2xl font-bold text-[var(--foreground)] mb-8">Enterprise proof</h2>
          <div className="flex flex-wrap gap-4 mb-8">
            {["AWS", "Azure", "GCP"].map((cloud) => (
              <span key={cloud} className="inline-flex items-center gap-2 rounded border border-[var(--border-glass)] bg-[var(--surface)]/60 px-4 py-2 text-sm font-mono text-[var(--foreground-secondary)]">
                {cloud}
                <span className="text-[10px] uppercase text-[var(--compliance)]">Certified Partner</span>
              </span>
            ))}
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-8">
            <div className="glass rounded-lg p-4 text-center">
              <p className="text-2xl font-mono font-semibold text-[var(--compliance)]">0ms</p>
              <p className="text-sm text-[var(--foreground-secondary)]">Latency overhead</p>
            </div>
            <div className="glass rounded-lg p-4 text-center">
              <p className="text-2xl font-mono font-semibold text-[var(--compliance)]">100K</p>
              <p className="text-sm text-[var(--foreground-secondary)]">RPS throughput</p>
            </div>
            <div className="glass rounded-lg p-4 text-center">
              <p className="text-2xl font-mono font-semibold text-[var(--compliance)]">99.999%</p>
              <p className="text-sm text-[var(--foreground-secondary)]">Uptime</p>
            </div>
          </div>
          <p className="text-sm text-[var(--muted)] mb-2">Live policy enforcement (verifiable):</p>
          <CodeSnippet
            showPrompt
            code={`curl -X POST "\${UNBLOCKLLM_BASE_URL}/v1/chat/completions" \\
  -H "Authorization: Bearer \${OPENAI_API_KEY}" \\
  -d '{"model":"gpt-4o-mini","messages":[{"role":"user","content":"My SSN is 123-45-6789."}]}'
# → 400 or redacted per policy`}
          />
          <p className="mt-2 text-xs text-[var(--muted)]">
            <a href="https://github.com/unblockllm/unblockllm/blob/main/docs/MODEL_BENCHMARK_REPORT.md" target="_blank" rel="noopener noreferrer" className="text-[var(--compliance)] hover:underline cursor-pointer">Benchmark reports</a>
          </p>
        </div>
      </section>

      {/* Final CTA */}
      <section className="py-24 px-4 sm:px-6 border-t border-[var(--border-glass)] w-full">
        <div className="w-full max-w-6xl mx-auto min-w-0">
          <h2 className="text-2xl sm:text-3xl font-bold text-[var(--foreground)]">Your LLM Traffic Deserves Zero Trust</h2>
          <p className="mt-3 text-[var(--foreground-secondary)] max-w-xl">Deploy in 8 minutes — no code changes to your app.</p>
          <div className="mt-8 flex flex-wrap gap-4">
            <a href="https://github.com/unblockllm/unblockllm/blob/main/docs/ARCHITECTURE.md" target="_blank" rel="noopener noreferrer" className="inline-flex items-center gap-2 rounded-lg bg-[var(--compliance)] px-5 py-2.5 text-sm font-medium text-[var(--background)] hover:opacity-90 transition-opacity cursor-pointer">
              View Architecture
            </a>
            <a href="https://github.com/unblockllm/unblockllm" target="_blank" rel="noopener noreferrer" className="inline-flex items-center gap-2 rounded-lg border border-[var(--border-glass)] bg-transparent px-5 py-2.5 text-sm font-medium text-[var(--foreground)] hover:bg-[var(--surface)] transition-colors cursor-pointer">
              <IconGitHub className="w-5 h-5" /> View GitHub
            </a>
          </div>
        </div>
      </section>

      {/* Contact — visible email + mailto + copy so CTA works everywhere */}
      <section id="contact" className="py-20 px-4 sm:px-6 border-t border-[var(--border-glass)] w-full scroll-mt-20">
        <div className="w-full max-w-6xl mx-auto min-w-0">
          <h2 className="text-2xl font-bold text-[var(--foreground)]">Contact</h2>
          <p className="mt-2 text-[var(--foreground-secondary)] max-w-xl">For enterprise inquiries, support, or partnerships.</p>
          <div className="mt-6 flex flex-wrap items-center gap-3">
            <span className="font-mono text-sm text-[var(--foreground)]" aria-label="Email address">{CONTACT_EMAIL}</span>
            {/* <EmailUsButton email={CONTACT_EMAIL} /> */}
            <CopyEmailButton email={CONTACT_EMAIL} />
          </div>
        </div>
      </section>

      <footer className="border-t border-[var(--border-glass)] py-8 px-4 sm:px-6 w-full">
        <div className="w-full max-w-6xl mx-auto flex flex-col md:flex-row justify-between items-center gap-4 min-w-0">
          <div className="flex flex-col sm:flex-row items-center gap-4 sm:gap-6 order-2 md:order-1">
            <span className="text-sm text-[var(--muted)] font-mono">© 2026 UnblockLLM</span>
            <div className="flex items-center gap-4">
              <a href="https://github.com/unblockllm/unblockllm" target="_blank" rel="noopener noreferrer" className="text-[var(--foreground-secondary)] hover:text-[var(--compliance)] transition-colors cursor-pointer" aria-label="GitHub">
                <IconGitHub className="w-5 h-5" />
              </a>
              <Link href="/docs" className="text-sm text-[var(--foreground-secondary)] hover:text-[var(--compliance)] transition-colors cursor-pointer">Docs</Link>
              {/* <Link href="/#contact" className="text-sm text-[var(--foreground-secondary)] hover:text-[var(--compliance)] transition-colors cursor-pointer">Contact</Link> */}
            </div>
          </div>
          <span className="text-sm text-[var(--muted)] order-1 md:order-2">No PII in logs. Ever.</span>
        </div>
      </footer>
    </div>
  );
}
