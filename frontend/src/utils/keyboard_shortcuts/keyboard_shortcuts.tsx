export type ModifierKey = "ctrl" | "shift" | "alt" | "meta";

export type KeyboardShortcut = {
  key: string;
  modifiers?: ModifierKey[];
  description: string;
  preventDefault?: boolean;
};

export const KEYBOARD_SHORTCUTS = {
  EXECUTE_NOTEBOOK_CELL: {
    key: "Enter",
    modifiers: ["meta"],
    description: "Execute the notebook cell to generate results",
    preventDefault: true,
  },
  OPEN_COMMAND_MENU: {
    key: "k",
    modifiers: ["meta"],
    description: "Open the command menu",
    preventDefault: true,
  },
  ADD_MARKDOWN_CELL: {
    key: "m",
    modifiers: ["meta"],
    description: "Add markdown cell",
    preventDefault: true,
  },
  ADD_SQL_CELL: {
    key: "G",
    modifiers: ["meta"],
    description: "Add SQL cell",
    preventDefault: true,
  },
  ADD_CHART_CELL: {
    key: "c",
    modifiers: ["meta"],
    description: "Add chart cell",
    preventDefault: true,
  },
  MOVE_CELL_UP: {
    key: "ArrowUp",
    modifiers: ["meta"],
    description: "Move cell up",
    preventDefault: true,
  },
  MOVE_CELL_TOP: {
    key: "ArrowUp",
    modifiers: ["meta", "shift"],
    description: "Move cell to top",
    preventDefault: true,
  },
  MOVE_CELL_DOWN: {
    key: "ArrowDown",
    modifiers: ["meta"],
    description: "Move cell down",
    preventDefault: true,
  },
  MOVE_CELL_BOTTOM: {
    key: "ArrowDown",
    modifiers: ["meta", "shift"],
    description: "Move cell to bottom",
    preventDefault: true,
  },
  DELETE_CELL: {
    key: "Backspace",
    modifiers: ["meta"],
    description: "Delete cell",
    preventDefault: true,
  },
} as const satisfies Record<string, KeyboardShortcut>;

export const NOTEBOOK_CELL_KEYBOARD_SHORTCUTS = [
  KEYBOARD_SHORTCUTS.EXECUTE_NOTEBOOK_CELL,
];

export function matchesAnyShortcut(
  event: KeyboardEvent,
  shortcuts: KeyboardShortcut[]
): boolean {
  return shortcuts.some((shortcut) => matchesShortcut(event, shortcut));
}

export function matchesShortcut(
  event: KeyboardEvent,
  shortcut: KeyboardShortcut
): boolean {
  const keyMatches = event.key.toLowerCase() === shortcut.key.toLowerCase();

  if (!keyMatches) return false;

  const modifiers = shortcut.modifiers || [];
  const ctrlMatch = modifiers.includes("ctrl") ? event.ctrlKey : !event.ctrlKey;
  const shiftMatch = modifiers.includes("shift")
    ? event.shiftKey
    : !event.shiftKey;
  const altMatch = modifiers.includes("alt") ? event.altKey : !event.altKey;
  const metaMatch = modifiers.includes("meta") ? event.metaKey : !event.metaKey;

  return ctrlMatch && shiftMatch && altMatch && metaMatch;
}

export function getShortcutDisplayString(shortcut: KeyboardShortcut): string {
  const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
  const modifiers = shortcut.modifiers || [];

  const modifierSymbols = modifiers.map((mod) => {
    switch (mod) {
      case "meta":
        return isMac ? "⌘" : "Ctrl";
      case "ctrl":
        return isMac ? "⌃" : "Ctrl";
      case "shift":
        return isMac ? "⇧" : "Shift";
      case "alt":
        return isMac ? "⌥" : "Alt";
      default:
        return mod;
    }
  });

  const keyDisplay = shortcut.key === " " ? "Space" : shortcut.key;
  return [...modifierSymbols, keyDisplay].join(isMac ? "" : "+");
}

export function handleKeyboardShortcut(
  shortcut: KeyboardShortcut,
  callback: (event: KeyboardEvent) => void,
  enabled = true
) {
  const handleKeyDown = (event: KeyboardEvent) => {
    event.stopPropagation();
    if (!enabled) return;

    if (matchesShortcut(event, shortcut)) {
      if (shortcut.preventDefault) {
        event.preventDefault();
      }
      callback(event);
    }
  };

  return handleKeyDown;
}
