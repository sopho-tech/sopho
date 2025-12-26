import * as RadixToolbar from "@radix-ui/react-toolbar";

type ToolbarProps = {
  children: React.ReactNode;
  loop?: boolean;
  className?: string;
} & Omit<
  React.ComponentPropsWithoutRef<typeof RadixToolbar.Root>,
  "className" | "loop"
>;

export function Toolbar({ children, loop, className, ...props }: ToolbarProps) {
  return (
    <RadixToolbar.Root className={className} loop={loop} {...props}>
      {children}
    </RadixToolbar.Root>
  );
}

Toolbar.Button = RadixToolbar.Button;
Toolbar.Separator = RadixToolbar.Separator;
Toolbar.ToggleGroup = RadixToolbar.ToggleGroup;
Toolbar.ToggleItem = RadixToolbar.ToggleItem;
Toolbar.Link = RadixToolbar.Link;
