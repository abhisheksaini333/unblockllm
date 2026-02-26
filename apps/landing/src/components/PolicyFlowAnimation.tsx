"use client";

import { useEffect, useRef, useCallback } from "react";

const AUDIT_ENTRIES_MAX = 3;
const GRID_SPACING = 48;
const GRID_STROKE = "rgba(56, 189, 248, 0.04)";
const AUDIT_TTL_MS = 4000;
const HERO_RATIO = 0.88;
const INBOUND_SPEED = 0.08;
const INBOUND_SPAWN_MS = 450;
const INBOUND_MAX = 6;
const GATE_THRESHOLD = 18;
const OUTCOME_DURATION_MS = 1200;
const OUTCOME_SPEED = 0.12;
const LANE_COUNT = 3;

type OutcomeKind = "allowed" | "blocked" | "monitored" | "masked";

interface AuditEntry {
  text: string;
  kind: OutcomeKind;
  x: number;
  y: number;
  createdAt: number;
}

interface InboundDot {
  x: number;
  lane: number;
}

interface OutcomeMarker {
  kind: OutcomeKind;
  x: number;
  lane: number;
  createdAt: number;
}

const AUDIT_TEMPLATES: { text: string; kind: OutcomeKind }[] = [
  { text: "BLOCKED  credit_card=**** @ ", kind: "blocked" },
  { text: "MASKED   email=****       @ ", kind: "masked" },
  { text: "MONITORED GITHUB_TOKEN   @ ", kind: "monitored" },
  { text: "ALLOWED   query='weather'  @ ", kind: "allowed" },
];

export function PolicyFlowAnimation({
  policyEngineHighlight = false,
  className = "",
}: {
  policyEngineHighlight?: boolean;
  className?: string;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const auditRef = useRef<AuditEntry[]>([]);
  const inboundRef = useRef<InboundDot[]>([]);
  const outcomesRef = useRef<OutcomeMarker[]>([]);
  const lastInboundSpawnRef = useRef<number>(0);
  const auditIndexRef = useRef(0);
  const lastTimeRef = useRef<number>(0);
  const policyPulseRef = useRef(0);

  const resizeCanvas = useCallback((canvas: HTMLCanvasElement) => {
    const dpr = Math.min(window.devicePixelRatio ?? 1, 2);
    const w = window.innerWidth;
    const h = window.innerHeight;
    canvas.width = w * dpr;
    canvas.height = h * dpr;
    canvas.style.width = `${w}px`;
    canvas.style.height = `${h}px`;
    return { w, h, dpr, ctx: canvas.getContext("2d") };
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reducedMotion) return;

    let rafId: number;
    let { w, h, dpr } = resizeCanvas(canvas);

    function drawGrid(c: CanvasRenderingContext2D, width: number, height: number) {
      c.save();
      c.scale(dpr, dpr);
      c.strokeStyle = GRID_STROKE;
      c.lineWidth = 0.5;
      for (let x = 0; x <= width; x += GRID_SPACING) {
        c.beginPath();
        c.moveTo(x, 0);
        c.lineTo(x, height);
        c.stroke();
      }
      for (let y = 0; y <= height; y += GRID_SPACING) {
        c.beginPath();
        c.moveTo(0, y);
        c.lineTo(width, y);
        c.stroke();
      }
      c.restore();
    }

    function drawPolicyGate(
      c: CanvasRenderingContext2D,
      width: number,
      height: number,
      pulse: number
    ) {
      const cx = width * 0.5;
      const bandW = 80;
      c.save();
      c.scale(dpr, dpr);

      const breath = 0.03 + 0.02 * Math.sin(performance.now() / 1400);
      const gradient = c.createLinearGradient(cx - bandW, 0, cx + bandW, 0);
      gradient.addColorStop(0, "rgba(15, 23, 42, 0)");
      gradient.addColorStop(0.45, `rgba(22, 163, 74, ${0.05 * (1 + pulse) + breath})`);
      gradient.addColorStop(0.5, `rgba(22, 163, 74, ${0.07 * (1 + pulse) + breath})`);
      gradient.addColorStop(0.55, `rgba(22, 163, 74, ${0.05 * (1 + pulse) + breath})`);
      gradient.addColorStop(1, "rgba(15, 23, 42, 0)");

      c.fillStyle = gradient;
      c.fillRect(cx - bandW, 0, bandW * 2, height);

      c.strokeStyle = `rgba(22, 163, 74, ${0.1 + pulse * 0.06})`;
      c.lineWidth = 1;
      c.setLineDash([6, 8]);
      c.beginPath();
      c.moveTo(cx, 0);
      c.lineTo(cx, height);
      c.stroke();
      c.setLineDash([]);
      c.restore();
    }

    function getLaneY(lane: number, height: number): number {
      const heroH = height * HERO_RATIO;
      const step = heroH / (LANE_COUNT + 1);
      return step * (lane + 1);
    }

    function drawInbound(
      c: CanvasRenderingContext2D,
      width: number,
      height: number,
      cx: number
    ) {
      const heroH = height * HERO_RATIO;
      c.save();
      c.scale(dpr, dpr);
      inboundRef.current.forEach((dot) => {
        const y = getLaneY(dot.lane, height);
        const alpha = 0.35 + 0.15 * Math.sin((dot.x / 20) * Math.PI);
        c.fillStyle = `rgba(148, 163, 184, ${alpha})`;
        c.beginPath();
        c.arc(dot.x, y, 3, 0, Math.PI * 2);
        c.fill();
      });
      c.restore();
    }

    function drawOutcomes(
      c: CanvasRenderingContext2D,
      width: number,
      height: number,
      now: number,
      cx: number
    ) {
      const heroH = height * HERO_RATIO;
      c.save();
      c.scale(dpr, dpr);
      c.font = "bold 14px system-ui, sans-serif";
      c.textAlign = "center";
      c.textBaseline = "middle";

      outcomesRef.current
        .filter((o) => now - o.createdAt < OUTCOME_DURATION_MS)
        .forEach((o) => {
          const age = now - o.createdAt;
          const progress = age / OUTCOME_DURATION_MS;
          const y = getLaneY(o.lane, height);
          const alpha = 1 - progress * progress;
          if (alpha <= 0) return;

          const isBlocked = o.kind === "blocked";
          const x = isBlocked ? cx + 20 : cx + 20 + age * 0.12;

          if (o.kind === "allowed") {
            c.fillStyle = `rgba(22, 163, 74, ${alpha})`;
            c.fillText("✓", x, y);
          } else if (o.kind === "monitored") {
            c.fillStyle = `rgba(234, 179, 8, ${alpha})`;
            c.fillText("●", x, y);
          } else if (o.kind === "masked") {
            c.fillStyle = `rgba(100, 116, 139, ${alpha})`;
            c.fillText("█", x, y);
          } else {
            c.fillStyle = `rgba(220, 38, 38, ${alpha})`;
            c.fillText("✗", x, y);
          }
        });
      c.restore();
    }

    function drawAuditLog(c: CanvasRenderingContext2D, width: number, height: number, now: number) {
      const entries = auditRef.current
        .filter((e) => now - e.createdAt < AUDIT_TTL_MS)
        .slice(-AUDIT_ENTRIES_MAX);
      if (entries.length === 0) return;
      c.save();
      c.scale(dpr, dpr);
      c.font = "11px ui-monospace, monospace";
      const left = width - 320;
      const top = height - 72;
      entries.forEach((e, i) => {
        const age = now - e.createdAt;
        const alpha = age > AUDIT_TTL_MS - 500 ? (AUDIT_TTL_MS - age) / 500 : 1;
        const borderColor =
          e.kind === "allowed"
            ? "#16a34a"
            : e.kind === "blocked"
              ? "#dc2626"
              : e.kind === "masked"
                ? "#64748b"
                : "#eab308";
        const rowY = top + i * 20;
        c.fillStyle = `rgba(15, 23, 42, ${0.92 * alpha})`;
        c.strokeStyle = `rgba(100, 116, 139, 0.4)`;
        c.lineWidth = 1;
        c.fillRect(left, rowY - 12, 308, 18);
        c.strokeRect(left, rowY - 12, 308, 18);
        c.fillStyle = borderColor;
        c.fillRect(left, rowY - 12, 3, 18);
        c.fillStyle = `rgba(203, 213, 225, ${alpha})`;
        c.textAlign = "left";
        c.fillText(e.text, left + 10, rowY);
      });
      c.restore();
    }

    function animate(now: number) {
      if (!canvas || !ctx) {
        rafId = requestAnimationFrame(animate);
        return;
      }
      const dt = lastTimeRef.current ? Math.min(now - lastTimeRef.current, 50) : 16;
      lastTimeRef.current = now;

      w = window.innerWidth;
      h = window.innerHeight;
      if (canvas.width !== w * dpr || canvas.height !== h * dpr) {
        const resized = resizeCanvas(canvas);
        w = resized.w;
        h = resized.h;
        dpr = resized.dpr;
      }

      const cx = w * 0.5;
      const heroH = h * HERO_RATIO;

      if (now - lastInboundSpawnRef.current > INBOUND_SPAWN_MS && inboundRef.current.length < INBOUND_MAX) {
        lastInboundSpawnRef.current = now;
        inboundRef.current.push({
          x: 40,
          lane: Math.floor(Math.random() * LANE_COUNT),
        });
      }

      inboundRef.current.forEach((dot) => {
        dot.x += INBOUND_SPEED * dt;
      });

      const hitGate = inboundRef.current.filter((dot) => dot.x >= cx - GATE_THRESHOLD);
      hitGate.forEach((dot) => {
        const t = AUDIT_TEMPLATES[auditIndexRef.current % AUDIT_TEMPLATES.length];
        auditIndexRef.current += 1;
        auditRef.current.push({
          text: t.text + `${Math.round(now % 100000)}ms`,
          kind: t.kind,
          x: cx + 80,
          y: h - 72,
          createdAt: now,
        });
        outcomesRef.current.push({
          kind: t.kind,
          x: cx + 20,
          lane: dot.lane,
          createdAt: now,
        });
      });
      auditRef.current = auditRef.current.slice(-AUDIT_ENTRIES_MAX * 2);

      inboundRef.current = inboundRef.current.filter((dot) => dot.x < cx - GATE_THRESHOLD);

      outcomesRef.current = outcomesRef.current.filter(
        (o) => now - o.createdAt < OUTCOME_DURATION_MS
      );

      policyPulseRef.current = policyEngineHighlight
        ? Math.min(1, policyPulseRef.current + dt / 200)
        : Math.max(0, policyPulseRef.current - dt / 150);
      const pulse = 0.25 + 0.35 * Math.sin(now / 220) * (policyEngineHighlight ? 1 : 0.2);

      ctx.clearRect(0, 0, canvas.width, canvas.height);
      drawGrid(ctx, w, h);
      drawInbound(ctx, w, h, cx);
      drawPolicyGate(ctx, w, h, policyPulseRef.current * pulse);
      drawOutcomes(ctx, w, h, now, cx);
      drawAuditLog(ctx, w, h, now);

      rafId = requestAnimationFrame(animate);
    }

    rafId = requestAnimationFrame(animate);

    const onResize = () => {
      if (canvas) resizeCanvas(canvas);
    };
    window.addEventListener("resize", onResize);

    return () => {
      window.removeEventListener("resize", onResize);
      cancelAnimationFrame(rafId);
    };
  }, [resizeCanvas, policyEngineHighlight]);

  return (
    <canvas
      ref={canvasRef}
      className={`fixed inset-0 -z-10 ${className}`}
      aria-hidden="true"
    />
  );
}
