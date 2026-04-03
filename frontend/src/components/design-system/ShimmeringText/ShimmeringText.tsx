import { motion, type Variants } from "motion/react";
import { useCallback } from "react";
import styles from "src/components/design-system/ShimmeringText/ShimmeringText.module.css";

type FontSize = "xs" | "sm" | "base" | "lg" | "xl" | "2xl" | "3xl" | "4xl";

type ShimmeringTextProps = {
  text: string;
  fontSize?: FontSize;
  duration?: number;
  baseColor?: string;
  shimmerColor?: string;
  className?: string;
};

export function ShimmeringText({
  text,
  fontSize,
  duration = 1,
  baseColor,
  shimmerColor,
  className,
}: ShimmeringTextProps) {
  const charCount = text.length;

  const createCharVariants = useCallback(
    (charIndex: number): Variants => ({
      animate: {
        color: ["var(--color)", "var(--shimmer-color)", "var(--color)"],
        transition: {
          duration,
          repeat: Infinity,
          repeatType: "loop",
          repeatDelay: charCount * 0.05,
          delay: (charIndex * duration) / charCount,
          ease: "easeInOut",
        },
      },
    }),
    [duration, charCount],
  );

  const containerClass = [
    styles.container,
    fontSize && styles[`fontSize-${fontSize}`],
    className,
  ]
    .filter(Boolean)
    .join(" ");

  const inlineStyle = {
    ...(baseColor && { "--color": baseColor }),
    ...(shimmerColor && { "--shimmer-color": shimmerColor }),
  } as React.CSSProperties;

  return (
    <span key={text} className={containerClass} style={inlineStyle}>
      {text.split("").map((char, i) => (
        <motion.span
          key={i}
          className={styles.char}
          variants={createCharVariants(i)}
          animate="animate"
        >
          {char}
        </motion.span>
      ))}
    </span>
  );
}
