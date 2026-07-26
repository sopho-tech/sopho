import {
  useEditor,
  EditorContent,
  EditorContext,
  type Editor,
} from "@tiptap/react";
import { useMemo, useImperativeHandle, type ReactNode, type Ref } from "react";
import Blockquote from "@tiptap/extension-blockquote";
import Bold from "@tiptap/extension-bold";
import BulletList from "@tiptap/extension-bullet-list";
import Code from "@tiptap/extension-code";
import CodeBlock from "@tiptap/extension-code-block";
import Document from "@tiptap/extension-document";
import HardBreak from "@tiptap/extension-hard-break";
import TiptapHeading from "@tiptap/extension-heading";
import HorizontalRule from "@tiptap/extension-horizontal-rule";
import Italic from "@tiptap/extension-italic";
import Link from "@tiptap/extension-link";
import ListItem from "@tiptap/extension-list-item";
import ListKeymap from "@tiptap/extension-list-keymap";
import OrderedList from "@tiptap/extension-ordered-list";
import Paragraph from "@tiptap/extension-paragraph";
import Strike from "@tiptap/extension-strike";
import Text_ from "@tiptap/extension-text";
import Underline from "@tiptap/extension-underline";
import {
  Dropcursor,
  Gapcursor,
  Placeholder,
  TrailingNode,
  UndoRedo,
} from "@tiptap/extensions";
import { Box, Flex, IconButton } from "src/components/design-system";
import {
  EditorNodeName,
  SlashCommandExtension,
  isSlashSuggestionActive,
} from "./SlashCommandExtension";
import type { MessageSegment } from "src/components/ConversationalAnalytics/dto";
import styles from "./MessageComposer.module.css";

export type MessageComposerHandle = {
  setText: (text: string) => void;
};

type MessageComposerProps = {
  placeholder: string;
  onSubmit: (segments: MessageSegment[]) => void;
  disabled: boolean;
  disabledTooltip?: string;
  enabledTooltip?: string;
  slotLeft?: ReactNode;
  ref?: Ref<MessageComposerHandle>;
};

const extractSegments = (editor: Editor): MessageSegment[] => {
  const segments: MessageSegment[] = [];
  editor.state.doc.descendants((node) => {
    if (node.type.name === EditorNodeName.SlashCommand) {
      segments.push({ type: "COMMAND", name: node.attrs.commandName });
    } else if (node.isText && node.text) {
      segments.push({ type: "TEXT", text: node.text });
    } else if (node.type.name === "paragraph" && segments.length > 0) {
      const last = segments[segments.length - 1];
      if (last.type === "TEXT" && !last.text.endsWith("\n")) {
        last.text += "\n";
      }
    }
  });
  return segments;
};

export function MessageComposer({
  placeholder,
  onSubmit,
  disabled,
  disabledTooltip,
  enabledTooltip = "Send",
  slotLeft,
  ref,
}: MessageComposerProps) {
  const editor = useEditor({
    extensions: [
      Document,
      Paragraph,
      Text_,
      Blockquote,
      Bold,
      BulletList,
      Code,
      CodeBlock,
      Dropcursor,
      Gapcursor,
      HardBreak,
      TiptapHeading,
      HorizontalRule,
      Italic,
      Link,
      ListItem,
      ListKeymap,
      OrderedList,
      Strike,
      TrailingNode,
      Underline,
      UndoRedo,
      SlashCommandExtension,
      Placeholder.configure({
        placeholder,
        emptyEditorClass: "is-editor-empty",
      }),
    ],
    content: "",
    editorProps: {
      attributes: {
        class: styles.tiptap,
      },
      handleKeyDown: (_view, event) => {
        if (event.key === "Enter" && !event.shiftKey) {
          if (editor && isSlashSuggestionActive(editor)) return false;
          event.preventDefault();
          if (disabled || !editor) return true;
          const segments = extractSegments(editor);
          const text = editor.getText().trim();
          const hasCommand = segments.some((s) => s.type === "COMMAND");
          if (!text && !hasCommand) return true;
          editor.commands.clearContent(true);
          onSubmit(segments);
          return true;
        }
        return false;
      },
    },
  });

  useImperativeHandle(
    ref,
    () => ({
      setText: (text: string) => {
        if (!editor) return;
        editor.commands.setContent(text);
        editor.commands.focus("end");
      },
    }),
    [editor],
  );

  const providerValue = useMemo(() => ({ editor }), [editor]);

  const handleClickSend = () => {
    if (disabled || !editor) return;
    const segments = extractSegments(editor);
    const text = editor.getText().trim();
    const hasCommand = segments.some((s) => s.type === "COMMAND");
    if (!text && !hasCommand) return;
    editor.commands.clearContent(true);
    onSubmit(segments);
  };

  return (
    <EditorContext.Provider value={providerValue}>
      <Flex
        direction="column"
        borderRadius="2xl"
        overflow="hidden"
        sx={{ boxShadow: "var(--shadow-border-medium)" }}
      >
        <Flex flex="grow" width="100%" sx={{ minWidth: 0, minHeight: "70px" }}>
          <EditorContent editor={editor} className={styles.editorContent} />
        </Flex>
        <Flex
          justifyContent="space-between"
          alignItems="center"
          gap="sm"
          paddingTop="lg"
          paddingBottom="sm"
          paddingX="lg"
          sx={{
            flexShrink: 0,
            backgroundColor: "var(--color-background)",
          }}
        >
          {slotLeft}
          <Box sx={{ opacity: disabled ? 0.5 : 1, marginLeft: "auto" }}>
            <IconButton
              type="arrow_up"
              backgroundColor="accent"
              iconColor="white"
              tooltip={{
                text: disabled ? (disabledTooltip ?? "") : enabledTooltip,
                direction: "top",
              }}
              onClick={handleClickSend}
              size="md"
              disabled={disabled}
            />
          </Box>
        </Flex>
      </Flex>
    </EditorContext.Provider>
  );
}
