import * as RadixHoverCard from "@radix-ui/react-hover-card";
import styles from "src/components/design-system/HoverCard/HoverCard.module.css";

export type HoverCardRootProps = {
  children: React.ReactNode;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  openDelay?: number;
  closeDelay?: number;
};

const HoverCardRoot = ({
  children,
  open,
  defaultOpen,
  onOpenChange,
  openDelay = 250,
  closeDelay = 150,
}: HoverCardRootProps) => (
  <RadixHoverCard.Root
    open={open}
    defaultOpen={defaultOpen}
    onOpenChange={onOpenChange}
    openDelay={openDelay}
    closeDelay={closeDelay}
  >
    {children}
  </RadixHoverCard.Root>
);

HoverCardRoot.displayName = "HoverCard";

export type HoverCardTriggerProps = {
  children: React.ReactNode;
  asChild?: boolean;
  className?: string;
};

const HoverCardTrigger = ({
  children,
  asChild = true,
  className,
}: HoverCardTriggerProps) => (
  <RadixHoverCard.Trigger asChild={asChild} className={className}>
    {children}
  </RadixHoverCard.Trigger>
);

HoverCardTrigger.displayName = "HoverCard.Trigger";

export type HoverCardContentProps = {
  children: React.ReactNode;
  side?: "top" | "right" | "bottom" | "left";
  align?: "start" | "center" | "end";
  sideOffset?: number;
  className?: string;
};

const HoverCardContent = ({
  children,
  side = "bottom",
  align = "end",
  sideOffset = 6,
  className,
}: HoverCardContentProps) => (
  <RadixHoverCard.Portal>
    <RadixHoverCard.Content
      className={`${styles.content} ${className ?? ""}`}
      side={side}
      align={align}
      sideOffset={sideOffset}
      collisionPadding={8}
    >
      {children}
    </RadixHoverCard.Content>
  </RadixHoverCard.Portal>
);

HoverCardContent.displayName = "HoverCard.Content";

export const HoverCard = Object.assign(HoverCardRoot, {
  Trigger: HoverCardTrigger,
  Content: HoverCardContent,
});
