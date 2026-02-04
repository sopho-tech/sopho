/**
 * Breakpoint constants for responsive design
 *
 * These breakpoints use minimum width media queries:
 * @media (width >= {value}) { ... }
 */

export const breakpoints = {
  sm: {
    rem: "40rem",
    px: 640,
  },
  md: {
    rem: "48rem",
    px: 768,
  },
  lg: {
    rem: "64rem",
    px: 1024,
  },
  xl: {
    rem: "80rem",
    px: 1280,
  },
  "2xl": {
    rem: "96rem",
    px: 1536,
  },
} as const;

export type BreakpointName = keyof typeof breakpoints;

/**
 * Get breakpoint value in rem
 */
export function getBreakpointRem(breakpoint: BreakpointName): string {
  return breakpoints[breakpoint].rem;
}

/**
 * Get breakpoint value in pixels
 */
export function getBreakpointPx(breakpoint: BreakpointName): number {
  return breakpoints[breakpoint].px;
}

/**
 * Generate a min-width media query string
 * @example
 * mediaQuery("md") // "@media (width >= 48rem)"
 */
export function mediaQuery(breakpoint: BreakpointName): string {
  return `@media (width >= ${breakpoints[breakpoint].rem})`;
}
