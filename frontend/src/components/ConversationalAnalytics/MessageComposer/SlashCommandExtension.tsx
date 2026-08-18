import { Node, mergeAttributes, type Editor } from "@tiptap/core";
import { ReactNodeViewRenderer } from "@tiptap/react";
import Suggestion, {
  type SuggestionOptions,
  type SuggestionProps,
} from "@tiptap/suggestion";
import { PluginKey } from "@tiptap/pm/state";
import { createRoot, type Root } from "react-dom/client";
import { createElement, createRef } from "react";
import { SlashCommandBadge } from "./SlashCommandBadge";
import {
  SlashCommandList,
  type SlashCommandListHandle,
} from "./SlashCommandList";
import {
  findMatchingSlashCommands,
  type SlashCommand,
} from "src/constants/slash_commands";

export enum EditorNodeName {
  SlashCommand = "slashCommand",
}

export type SlashCommandExtensionOptions = {
  availableCommands: SlashCommand[];
};

const slashCommandPluginKey = new PluginKey(EditorNodeName.SlashCommand);

export const isSlashSuggestionActive = (
  editor: Editor,
  availableCommands: SlashCommand[],
): boolean => {
  const suggestionState = slashCommandPluginKey.getState(editor.state);
  if (!suggestionState?.active) return false;
  return (
    findMatchingSlashCommands(availableCommands, suggestionState.query).length >
    0
  );
};

const buildSuggestion = (
  availableCommands: SlashCommand[],
): Omit<SuggestionOptions<SlashCommand>, "editor"> => ({
  char: "/",
  pluginKey: slashCommandPluginKey,
  items: ({ query }) => findMatchingSlashCommands(availableCommands, query),
  command: ({ editor, range, props }) => {
    editor
      .chain()
      .focus()
      .insertContentAt(range, [
        {
          type: EditorNodeName.SlashCommand,
          attrs: { commandName: props.name },
        },
        { type: "text", text: " " },
      ])
      .run();
  },
  render: () => {
    let container: HTMLDivElement | null = null;
    let root: Root | null = null;
    const listRef = createRef<SlashCommandListHandle>();

    const renderList = (props: SuggestionProps<SlashCommand>) => {
      root?.render(
        createElement(SlashCommandList, {
          ref: listRef,
          items: props.items,
          command: props.command,
          clientRect: props.clientRect,
        }),
      );
    };

    return {
      onStart: (props) => {
        container = document.createElement("div");
        document.body.appendChild(container);
        root = createRoot(container);
        renderList(props);
      },
      onUpdate: (props) => renderList(props),
      onKeyDown: (props) => {
        if (props.event.key === "Escape") return true;
        return listRef.current?.onKeyDown(props.event) ?? false;
      },
      onExit: () => {
        root?.unmount();
        container?.remove();
        root = null;
        container = null;
      },
    };
  },
});

export const SlashCommandExtension = Node.create<SlashCommandExtensionOptions>({
  name: EditorNodeName.SlashCommand,
  group: "inline",
  inline: true,
  atom: true,
  selectable: true,

  addOptions() {
    return { availableCommands: [] };
  },

  addAttributes() {
    return {
      commandName: {
        default: "",
        parseHTML: (element) => element.getAttribute("data-slash-command"),
        renderHTML: (attributes) => ({
          "data-slash-command": attributes.commandName,
        }),
      },
    };
  },

  parseHTML() {
    return [{ tag: "span[data-slash-command]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return ["span", mergeAttributes(HTMLAttributes)];
  },

  addNodeView() {
    return ReactNodeViewRenderer(SlashCommandBadge);
  },

  addProseMirrorPlugins() {
    if (this.options.availableCommands.length === 0) return [];
    return [
      Suggestion({
        editor: this.editor,
        ...buildSuggestion(this.options.availableCommands),
      }),
    ];
  },
});
