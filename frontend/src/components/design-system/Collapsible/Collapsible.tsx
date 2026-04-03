import * as RadixCollapsible from "@radix-ui/react-collapsible";
import styles from "./Collapsible.module.css";

export type CollapsibleRootProps = {
  children: React.ReactNode;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  disabled?: boolean;
  className?: string;
};

const CollapsibleRoot = ({
  children,
  open,
  defaultOpen,
  onOpenChange,
  disabled,
  className,
}: CollapsibleRootProps) => (
  <RadixCollapsible.Root
    open={open}
    defaultOpen={defaultOpen}
    onOpenChange={onOpenChange}
    disabled={disabled}
    className={className}
  >
    {children}
  </RadixCollapsible.Root>
);

CollapsibleRoot.displayName = "Collapsible";

export type CollapsibleTriggerProps = {
  children: React.ReactNode;
  asChild?: boolean;
  className?: string;
};

const CollapsibleTrigger = ({
  children,
  asChild = true,
  className,
}: CollapsibleTriggerProps) => (
  <RadixCollapsible.Trigger
    asChild={asChild}
    className={asChild ? className : `${styles.trigger} ${className ?? ""}`}
  >
    {children}
  </RadixCollapsible.Trigger>
);

CollapsibleTrigger.displayName = "Collapsible.Trigger";

export type CollapsibleContentProps = {
  children: React.ReactNode;
  forceMount?: true;
  className?: string;
};

const CollapsibleContent = ({
  children,
  forceMount,
  className,
}: CollapsibleContentProps) => (
  <RadixCollapsible.Content
    forceMount={forceMount}
    className={`${styles.content} ${className ?? ""}`}
  >
    {children}
  </RadixCollapsible.Content>
);

CollapsibleContent.displayName = "Collapsible.Content";

export const Collapsible = Object.assign(CollapsibleRoot, {
  Trigger: CollapsibleTrigger,
  Content: CollapsibleContent,
});
