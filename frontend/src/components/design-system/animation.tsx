import { Variants, Transition } from "motion/react";

export enum EnterAnimation {
  BLUR = "BLUR",
  SLIDE_DOWN = "SLIDE_DOWN",
}

export enum Duration {
  VERY_FAST = 0.02,
  FAST = 0.2,
  NORMAL = 0.4,
  MEDIUM = 0.6,
  SLOW = 0.8,
}

export enum Scale {
  HOVER = 1.05,
  TAP = 0.9,
}

export const EASE = [0.4, 0, 0.2, 1] as const;

export const hoverHighlightTransition: Transition = {
  duration: Duration.VERY_FAST,
  ease: EASE,
};

export type AnimationType = {
  enter?: EnterAnimation;
};

export type AnimationProps = {
  animations?: AnimationType;
};

export const enterAnimationVariants: Record<EnterAnimation, Variants> = {
  [EnterAnimation.BLUR]: {
    initial: {
      filter: "blur(10px)",
      opacity: 0,
      y: -20,
    },
    animate: {
      filter: "blur(0px)",
      opacity: 1,
      y: 0,
    },
  },
  [EnterAnimation.SLIDE_DOWN]: {
    initial: {
      opacity: 0,
      y: -24,
    },
    animate: {
      opacity: 1,
      y: 0,
    },
  },
};

export const defaultEnterTransition: Transition = {
  duration: Duration.MEDIUM,
  ease: EASE,
};

export const layoutSpringTransition: Transition = {
  type: "spring",
  stiffness: 500,
  damping: 35,
};

export const rotateToggleTransition: Transition = {
  duration: 0.3,
  ease: [0.87, 0, 0.13, 1],
};

export const enterAnimationTransitions: Record<EnterAnimation, Transition> = {
  [EnterAnimation.BLUR]: {
    ...defaultEnterTransition,
    filter: {
      duration: Duration.SLOW,
      ease: EASE,
    },
  },
  [EnterAnimation.SLIDE_DOWN]: {
    ...defaultEnterTransition,
  },
};

export function getEnterAnimationVariants(
  animationType?: EnterAnimation,
): Variants | undefined {
  if (!animationType) return undefined;
  return enterAnimationVariants[animationType];
}

export function getEnterAnimationTransition(
  animationType?: EnterAnimation,
): Transition | undefined {
  if (!animationType) return undefined;
  return enterAnimationTransitions[animationType];
}

export function getEnterAnimationVariantsWithFallback(
  animationType?: EnterAnimation,
): Variants {
  const animationVariants = getEnterAnimationVariants(animationType);
  if (!animationVariants) {
    // When no animation is provided, create a hidden initial state
    return {
      initial: {
        opacity: 0,
      },
      animate: {
        opacity: 0,
      },
    };
  }
  return animationVariants;
}
