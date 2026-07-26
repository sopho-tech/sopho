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

export enum EditorNodeName {
  SlashCommand = "slashCommand",
}

export type SlashCommand = { name: string; description: string };

export const SLASH_COMMANDS: SlashCommand[] = [
  { name: "canvas", description: "Generate canvas for the conversation" },
];

const slashCommandPluginKey = new PluginKey(EditorNodeName.SlashCommand);

export const isSlashSuggestionActive = (editor: Editor): boolean =>
  slashCommandPluginKey.getState(editor.state)?.active ?? false;

const suggestion: Omit<SuggestionOptions<SlashCommand>, "editor"> = {
  char: "/",
  pluginKey: slashCommandPluginKey,
  items: ({ query }) =>
    SLASH_COMMANDS.filter((command) =>
      command.name.toLowerCase().startsWith(query.toLowerCase()),
    ),
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
};

export const SlashCommandExtension = Node.create({
  name: EditorNodeName.SlashCommand,
  group: "inline",
  inline: true,
  atom: true,
  selectable: true,

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
    return [Suggestion({ editor: this.editor, ...suggestion })];
  },
});
