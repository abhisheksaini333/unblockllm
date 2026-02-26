"use client";

import { useEffect } from "react";
import { motion } from "framer-motion";
import { IconShield } from "./icons";

const MOTION_DURATION = 0.3;

const steps = [
  {
    title: "Ingest",
    description: "Token-level inspection (no full payload decryption)",
    metrics: ["0ms decryption overhead", "100% token visibility"],
    codeSnippet: `curl -X POST https://api.unblockllm.com/v1/chat/completions \\
  -H "Authorization: Bearer YOUR_API_KEY" \\
  -d '{"messages":[{"role":"user","content":"Hello"}]}'`,
  },
  {
    title: "Policy Engine",
    description: "Custom rules: block credit_card if region=EU. Policy evaluated per token.",
    metrics: ["Per-token evaluation", "YAML or dashboard config"],
    codeSnippet: `block: [CREDIT_CARD]
if: request.headers["x-region"] == "EU"`,
  },
  {
    title: "Redact",
    description: "On-the-fly PII masking (retain token count). No payload size change.",
    metrics: ["Token count preserved", "Streaming redaction"],
    codeSnippet: `{"input":"SSN 123-45-6789","output":"SSN [REDACTED]"}`,
  },
  {
    title: "Audit",
    description: "Immutable logs for SOC 2 compliance. Metadata only, no PII.",
    metrics: ["90-day retention", "SOC 2 evidence"],
    codeSnippet: `{"request_id":"req_xyz","entity_count":1,"entity_types":["SSN"]}`,
  },
];

export function StepExplanation({
  activeStep,
  onStepChange,
}: {
  activeStep: number;
  onStepChange?: (index: number) => void;
}) {
  useEffect(() => {
    const handleScroll = () => {
      const sections = document.querySelectorAll<HTMLElement>(".step-section");
      const viewportMid = 160;
      let current = 0;
      sections.forEach((section, index) => {
        const rect = section.getBoundingClientRect();
        if (rect.top <= viewportMid && rect.bottom >= viewportMid) {
          current = index;
        }
      });
      onStepChange?.(current);
    };

    window.addEventListener("scroll", handleScroll, { passive: true });
    requestAnimationFrame(handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, [onStepChange]);

  return (
    <div className="space-y-8">
      {steps.map((step, index) => (
        <motion.div
          key={step.title}
          className={`step-section pl-4 border-l-2 transition-colors ${
            index === activeStep ? "border-[var(--compliance)]" : "border-transparent"
          }`}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: MOTION_DURATION }}
        >
          <h3 className="text-lg font-bold text-[var(--compliance)] mb-2 flex items-center gap-2">
            {index === 1 && <IconShield className="w-5 h-5 shrink-0 text-[var(--compliance)]" aria-hidden />}
            Step {index + 1} of {steps.length}: {step.title}
          </h3>
          <p className="text-[var(--foreground-secondary)] mb-3">{step.description}</p>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            {step.metrics.map((metric) => (
              <div key={metric} className="rounded-lg p-3 bg-[var(--surface)]/50 border border-[var(--border-glass)]">
                <p className="text-[var(--compliance)] font-mono text-sm">{metric}</p>
              </div>
            ))}
          </div>
        </motion.div>
      ))}
    </div>
  );
}

export { steps as HOW_IT_WORKS_STEPS };
