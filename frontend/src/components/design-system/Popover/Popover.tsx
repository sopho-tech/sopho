import * as RadixPopover from "@radix-ui/react-popover";
import styles from "src/components/design-system/Popover/Popover.module.css";

export type PopoverRootProps = {
  children: React.ReactNode;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  modal?: boolean;
};

const PopoverRoot = ({
  children,
  open,
  defaultOpen,
  onOpenChange,
  modal,
}: PopoverRootProps) => (
  <RadixPopover.Root
    open={open}
    defaultOpen={defaultOpen}
    onOpenChange={onOpenChange}
    modal={modal}
  >
    {children}
  </RadixPopover.Root>
);

PopoverRoot.displayName = "Popover";

export type PopoverTriggerProps = {
  children: React.ReactNode;
  asChild?: boolean;
  className?: string;
};

const PopoverTrigger = ({
  children,
  asChild = true,
  className,
}: PopoverTriggerProps) => (
  <RadixPopover.Trigger
    asChild={asChild}
    className={asChild ? className : `${styles.trigger} ${className ?? ""}`}
  >
    {children}
  </RadixPopover.Trigger>
);

PopoverTrigger.displayName = "Popover.Trigger";

export type PopoverContentProps = {
  children: React.ReactNode;
  side?: "top" | "right" | "bottom" | "left";
  align?: "start" | "center" | "end";
  sideOffset?: number;
  className?: string;
};

const PopoverContent = ({
  children,
  side = "bottom",
  align = "start",
  sideOffset = 6,
  className,
}: PopoverContentProps) => (
  <RadixPopover.Portal>
    <RadixPopover.Content
      className={`${styles.content} ${className ?? ""}`}
      side={side}
      align={align}
      sideOffset={sideOffset}
      collisionPadding={8}
    >
      {children}
    </RadixPopover.Content>
  </RadixPopover.Portal>
);

PopoverContent.displayName = "Popover.Content";

export const Popover = Object.assign(PopoverRoot, {
  Trigger: PopoverTrigger,
  Content: PopoverContent,
});
