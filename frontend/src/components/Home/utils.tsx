import type { DropdownMenuItem } from "src/components/design-system";

export function createDropdownItems(
  onCreateCanvas: () => void,
  onCreateConnection: () => void
): DropdownMenuItem[] {
  return [
    {
      icon: "book",
      label: "Canvas",
      onClick: onCreateCanvas,
    },
    {
      icon: "swap_horiz",
      label: "Connection",
      onClick: onCreateConnection,
    },
  ];
}
