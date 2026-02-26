"use client";

import { ReactNode } from "react";
import { motion } from "framer-motion";
import { useInView } from "react-intersection-observer";

const MOTION_DURATION = 0.2;

interface ScrollTriggerProps {
  children: ReactNode;
  className?: string;
  once?: boolean;
  threshold?: number;
  delay?: number;
}

/** Wraps content with scroll-triggered reveal. Respects prefers-reduced-motion. */
export function ScrollTrigger({
  children,
  className = "",
  once = true,
  threshold = 0.1,
  delay = 0,
}: ScrollTriggerProps) {
  const [ref, inView] = useInView({ triggerOnce: once, threshold });

  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 12 }}
      animate={inView ? { opacity: 1, y: 0 } : {}}
      transition={{ duration: MOTION_DURATION, delay }}
      className={className}
    >
      {children}
    </motion.div>
  );
}
