import type { DropdownMenuItem } from "src/components/design-system";

export function createDropdownItems(
  onCreateCanvas: () => void,
  onCreateConnection: () => void,
  canCreateCanvas: boolean,
  canCreateConnection: boolean,
): DropdownMenuItem[] {
  return [
    ...(canCreateCanvas
      ? [
          {
            icon: "book" as const,
            label: "Canvas",
            onClick: onCreateCanvas,
          },
        ]
      : []),
    ...(canCreateConnection
      ? [
          {
            icon: "swap_horiz" as const,
            label: "Connection",
            onClick: onCreateConnection,
          },
        ]
      : []),
  ];
}
