import { useEffect, useImperativeHandle, useState, type Ref } from "react";
import { Box, DropdownMenu, Text } from "src/components/design-system";
import type { SlashCommand } from "src/constants/slash_commands";

export type SlashCommandListHandle = {
  onKeyDown: (event: KeyboardEvent) => boolean;
};

type SlashCommandListProps = {
  items: SlashCommand[];
  command: (item: SlashCommand) => void;
  clientRect?: (() => DOMRect | null) | null;
  ref?: Ref<SlashCommandListHandle>;
};

export function SlashCommandList({
  items,
  command,
  clientRect,
  ref,
}: SlashCommandListProps) {
  const [selectedIndex, setSelectedIndex] = useState(0);

  useEffect(() => setSelectedIndex(0), [items]);

  const selectItem = (index: number) => {
    const item = items[index];
    if (item) command(item);
  };

  useImperativeHandle(ref, () => ({
    onKeyDown: (event: KeyboardEvent) => {
      if (items.length === 0) return false;
      if (event.key === "ArrowUp") {
        setSelectedIndex((i) => (i + items.length - 1) % items.length);
        return true;
      }
      if (event.key === "ArrowDown") {
        setSelectedIndex((i) => (i + 1) % items.length);
        return true;
      }
      if (event.key === "Enter") {
        selectItem(selectedIndex);
        return true;
      }
      return false;
    },
  }));

  const rect = clientRect?.();
  if (!rect) return null;

  return (
    <DropdownMenu open modal={false}>
      <DropdownMenu.Trigger>
        <span
          style={{
            position: "fixed",
            top: rect.top,
            left: rect.left,
            width: 1,
            height: rect.height,
            opacity: 0,
            pointerEvents: "none",
          }}
        />
      </DropdownMenu.Trigger>
      <DropdownMenu.Content
        side="bottom"
        align="start"
        sideOffset={4}
        preventAutoFocus
      >
        {items.length === 0 ? (
          <Box paddingX="md" paddingY="sm">
            <Text as="span" color="subtle" fontSize="sm">
              No matching commands
            </Text>
          </Box>
        ) : (
          items.map((item, index) => (
            <DropdownMenu.ItemWithDescription
              key={item.name}
              label={`/${item.name}`}
              description={item.description}
              highlighted={index === selectedIndex}
              onClick={() => selectItem(index)}
            />
          ))
        )}
      </DropdownMenu.Content>
    </DropdownMenu>
  );
}
