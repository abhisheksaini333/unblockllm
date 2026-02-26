"use client";

import { useEffect, useRef } from "react";

/**
 * Token Stream Diagnostic background: grid + P/S/D token stream.
 * Security-focused, canvas-based. Honors prefers-reduced-motion.
 */
export function TokenStreamBackground() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const tokens = [
      { char: "P", color: "#DC2626" },
      { char: "S", color: "#16A34A" },
      { char: "D", color: "#94A3B8" },
    ];

    const resizeCanvas = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    };

    window.addEventListener("resize", resizeCanvas);
    resizeCanvas();

    const prefersReducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const w0 = window.innerWidth || 800;
    const h0 = window.innerHeight || 600;
    const tokenStream = Array.from({ length: prefersReducedMotion ? 20 : 100 }, () => {
      const token = tokens[Math.floor(Math.random() * tokens.length)];
      return {
        x: Math.random() * w0,
        y: Math.random() * h0,
        char: token.char,
        color: token.color,
        size: 12 + Math.random() * 8,
        speed: prefersReducedMotion ? 0 : 0.5 + Math.random() * 1.5,
        opacity: 0.3 + Math.random() * 0.5,
      };
    });

    let rafId: number;

    const animate = () => {
      const w = canvas.width;
      const h = canvas.height;
      if (!w || !h) {
        rafId = requestAnimationFrame(animate);
        return;
      }

      ctx.clearRect(0, 0, w, h);

      ctx.strokeStyle = "rgba(56, 189, 248, 0.05)";
      ctx.lineWidth = 0.5;
      for (let x = 0; x < w; x += 40) {
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, h);
        ctx.stroke();
      }
      for (let y = 0; y < h; y += 40) {
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(w, y);
        ctx.stroke();
      }

      tokenStream.forEach((token) => {
        token.x += token.speed;
        if (token.x > w) token.x = 0;

        const r = parseInt(token.color.slice(1, 3), 16);
        const g = parseInt(token.color.slice(3, 5), 16);
        const b = parseInt(token.color.slice(5, 7), 16);
        ctx.font = `bold ${token.size}px system-ui, sans-serif`;
        ctx.fillStyle = `rgba(${r},${g},${b},${token.opacity})`;
        ctx.fillText(token.char, token.x, token.y);
      });

      rafId = requestAnimationFrame(animate);
    };

    animate();

    return () => {
      window.removeEventListener("resize", resizeCanvas);
      cancelAnimationFrame(rafId);
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      className="fixed inset-0 -z-10"
      aria-hidden="true"
    />
  );
}
