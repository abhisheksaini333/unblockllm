"use client";

import { CodeSnippet } from "./CodeSnippet";

const skills = [
  {
    name: "skills/github",
    risk: "Exposes repo tokens in prompts",
    protection: "Blocks GITHUB_TOKEN in all skill inputs",
    terminalCommand: `$ npx skills add skills/github
→ [UNBLOCKLLM] Scanning skill dependencies...
→ [POLICY] Detected: GITHUB_TOKEN exposure risk
→ [ACTION] Injected redaction layer
→ [STATUS] ✅ Skill secured & installed`,
  },
  {
    name: "skills/claude",
    risk: "Leaks PII to Anthropic",
    protection: "Redacts names/emails in real-time",
    terminalCommand: `$ npx skills add skills/claude
→ [UNBLOCKLLM] Scanning skill dependencies...
→ [POLICY] Detected: PII in prompt risk
→ [ACTION] Injected redaction layer
→ [STATUS] ✅ Skill secured & installed`,
  },
  {
    name: "skills/websearch",
    risk: "Sends sensitive queries to Google",
    protection: "Scrubs query parameters pre-request",
    terminalCommand: `$ npx skills add skills/websearch
→ [UNBLOCKLLM] Scanning skill dependencies...
→ [POLICY] Detected: query parameter exposure risk
→ [ACTION] Injected redaction layer
→ [STATUS] ✅ Skill secured & installed`,
  },
];

export function SkillsIntegration() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-[var(--foreground)] mb-2">
          UnblockLLM + Agent Skills: Secure by Default
        </h2>
        <p className="text-[var(--foreground-secondary)] mb-6">
          Your Skills Stay Powerful, Now Privacy-Compliant
        </p>
      </div>

      <div className="rounded-lg p-4 border border-[var(--border-glass)] border-l-2 border-l-[var(--compliance)]/50 bg-[var(--surface)]/50">
        <div className="flex items-center gap-2 mb-2">
          <span className="text-[var(--compliance)] font-mono" aria-hidden>✓</span>
          <span className="text-sm text-[var(--foreground-secondary)]">Policy enforced</span>
        </div>
        <p className="text-[var(--foreground-secondary)]">
          Blocks <span className="font-mono text-[var(--compliance)]">GITHUB_TOKEN</span> in all skill inputs
        </p>
        <p className="text-sm text-[var(--muted)] mt-1">No code changes required</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {skills.map((skill) => (
          <div
            key={skill.name}
            className="rounded-xl p-6 border border-[var(--border-glass)] bg-[var(--surface)]/50"
          >
            <h3 className="text-[var(--compliance)] font-mono mb-4">{skill.name}</h3>
            <div className="space-y-3 mb-4">
              <div>
                <p className="text-[var(--muted)] text-sm mb-1">Risk:</p>
                <p className="text-[var(--foreground-secondary)] text-sm">{skill.risk}</p>
              </div>
              <div>
                <p className="text-[var(--muted)] text-sm mb-1">How we secure it:</p>
                <p className="text-[var(--compliance)] text-sm">{skill.protection}</p>
              </div>
            </div>
            <CodeSnippet code={skill.terminalCommand} showPrompt />
          </div>
        ))}
      </div>

      <p className="pt-2">
        <a
          href="/docs#skills-sh"
          className="inline-flex items-center gap-2 rounded border border-[var(--compliance)] px-3 py-1.5 text-sm font-mono text-[var(--compliance)] hover:bg-[var(--compliance)]/10 transition-colors cursor-pointer"
        >
          Official skills.sh Security Partner
        </a>
      </p>
    </div>
  );
}
