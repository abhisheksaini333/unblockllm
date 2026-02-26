"use client";

import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { useInView } from "react-intersection-observer";

interface LiveMetricProps {
  value: number;
  suffix?: string;
  prefix?: string;
  label: string;
  duration?: number;
  /** Green pulse when count completes */
  pulseOnComplete?: boolean;
}

export function LiveMetric({
  value,
  suffix = "%",
  prefix = "",
  label,
  duration = 1.2,
  pulseOnComplete = true,
}: LiveMetricProps) {
  const [ref, inView] = useInView({ triggerOnce: true, threshold: 0.3 });
  const [display, setDisplay] = useState(0);
  const [complete, setComplete] = useState(false);
  const step = value / (duration * 60);

  useEffect(() => {
    if (!inView) return;
    let start = 0;
    const timer = setInterval(() => {
      start += step;
      if (start >= value) {
        setDisplay(value);
        setComplete(true);
        clearInterval(timer);
      } else {
        setDisplay(Math.round(start * 10) / 10);
      }
    }, 1000 / 60);
    return () => clearInterval(timer);
  }, [inView, value, step, duration]);

  return (
    <motion.div
      ref={ref}
      className="font-mono"
      animate={pulseOnComplete && complete ? { scale: [1, 1.02, 1] } : {}}
      transition={{ duration: 0.2 }}
    >
      <span className="text-2xl font-semibold text-[var(--compliance)]">
        {prefix}{display}{suffix}
      </span>
      <span className="text-[var(--foreground-secondary)] ml-2 text-sm">{label}</span>
    </motion.div>
  );
}
