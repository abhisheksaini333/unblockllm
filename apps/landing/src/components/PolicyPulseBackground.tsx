"use client";

import { useEffect, useState, useRef } from "react";

const PULSE_RADIUS = 1.5; // viewBox units (0-100) so ~1.5% of viewport
const BLOCKED_PULSE_MS = 1200;
/** Corner positions as percentage so SVG can render without knowing pixel size */
const CORNERS = [
  { x: "2%", y: "2%", kind: "allowed" as const },
  { x: "98%", y: "2%", kind: "blocked" as const },
  { x: "2%", y: "98%", kind: "allowed" as const },
  { x: "98%", y: "98%", kind: "blocked" as const },
];

/**
 * Non-intrusive Policy Pulse background: grid + corner pulses in negative space only.
 * Zero overlap with content. Honors prefers-reduced-motion.
 */
export function PolicyPulseBackground({
  policyEngineActive = false,
  blockedPulseTrigger,
  /** When true, corner pulses run even before user scrolls to Step 2 (so animation is visible on load) */
  alwaysShowCornerPulses = true,
  className = "",
}: {
  policyEngineActive?: boolean;
  blockedPulseTrigger?: number;
  alwaysShowCornerPulses?: boolean;
  className?: string;
}) {
  const [reduceMotion, setReduceMotion] = useState(false);
  const [showBlockedPulse, setShowBlockedPulse] = useState(false);
  const [mounted, setMounted] = useState(false);
  const blockedTriggerRef = useRef<number | undefined>();

  useEffect(() => {
    setMounted(true);
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    const handleReduce = () => setReduceMotion(mq.matches);
    handleReduce();
    mq.addEventListener("change", handleReduce);
    return () => mq.removeEventListener("change", handleReduce);
  }, []);

  useEffect(() => {
    if (
      typeof blockedPulseTrigger === "number" &&
      blockedPulseTrigger > 0 &&
      blockedPulseTrigger !== blockedTriggerRef.current
    ) {
      blockedTriggerRef.current = blockedPulseTrigger;
      setShowBlockedPulse(true);
      const t = setTimeout(() => setShowBlockedPulse(false), BLOCKED_PULSE_MS);
      return () => clearTimeout(t);
    }
  }, [blockedPulseTrigger]);

  const showCornerPulses = (alwaysShowCornerPulses || policyEngineActive) && !reduceMotion && mounted;

  return (
    <>
      <style>{`
        .policy-pulse-circle {
          animation: policy-corner-pulse 800ms cubic-bezier(0.4, 0, 0.6, 1) infinite;
          transform-origin: center;
          will-change: opacity;
        }
        .policy-pulse-blocked {
          animation: policy-blocked-pulse 1200ms cubic-bezier(0.22, 1, 0.36, 1) forwards;
          transform-origin: center;
          will-change: opacity;
        }
        @keyframes policy-corner-pulse {
          0%, 100% { opacity: 0.25; }
          50% { opacity: 0.65; }
        }
        @keyframes policy-blocked-pulse {
          0% { opacity: 0; }
          15% { opacity: 0.65; }
          85% { opacity: 0.4; }
          100% { opacity: 0; }
        }
        @media (prefers-reduced-motion: reduce) {
          .policy-pulse-circle,
          .policy-pulse-blocked { animation: none !important; opacity: 0.12 !important; }
        }
      `}</style>
      <svg
        className={`fixed inset-0 pointer-events-none -z-[1] w-full h-full ${className}`}
        width="100%"
        height="100%"
        viewBox="0 0 100 100"
        preserveAspectRatio="none"
        aria-hidden="true"
      >
        <defs>
          <pattern
            id="policy-grid"
            width="4"
            height="4"
            patternUnits="userSpaceOnUse"
          >
            <path
              d="M 4 0 L 0 0 0 4"
              fill="none"
              stroke="rgba(56, 189, 248, 0.08)"
              strokeWidth="0.08"
            />
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#policy-grid)" />

        {showCornerPulses &&
          CORNERS.map((c, i) => (
            <g key={i}>
              <circle
                cx={c.x}
                cy={c.y}
                r={PULSE_RADIUS}
                fill={c.kind === "allowed" ? "rgba(22, 163, 74, 0.3)" : "rgba(220, 38, 38, 0.3)"}
                className="policy-pulse-circle"
              />
              <text
                x={c.x}
                y={c.y}
                textAnchor="middle"
                dominantBaseline="central"
                fill={c.kind === "allowed" ? "#16A34A" : "#DC2626"}
                fontSize="10"
                fontWeight="bold"
              >
                {c.kind === "allowed" ? "✓" : "✗"}
              </text>
            </g>
          ))}

        {showBlockedPulse && !reduceMotion && (
          <g>
            <circle
              cx="98%"
              cy="98%"
              r={2.25}
              fill="rgba(220, 38, 38, 0.25)"
              className="policy-pulse-blocked"
            />
            <text x="98%" y="98%" textAnchor="middle" dominantBaseline="central" fill="#DC2626" fontSize="10" fontWeight="bold">
              ✗
            </text>
          </g>
        )}
      </svg>
    </>
  );
}
