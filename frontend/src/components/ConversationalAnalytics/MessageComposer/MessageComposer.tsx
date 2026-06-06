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
import styles from "./MessageComposer.module.css";

export type MessageComposerHandle = {
  setText: (text: string) => void;
};

type MessageComposerProps = {
  placeholder: string;
  onSubmit: (text: string) => void;
  disabled: boolean;
  disabledTooltip?: string;
  enabledTooltip?: string;
  slotLeft?: ReactNode;
  ref?: Ref<MessageComposerHandle>;
};

const extractAndClear = (editor: Editor): string | null => {
  const text = editor.getText().trim();
  if (!text) return null;
  editor.commands.clearContent(true);
  return text;
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
          event.preventDefault();
          if (disabled || !editor) return true;
          const text = extractAndClear(editor);
          if (text !== null) onSubmit(text);
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
    const text = extractAndClear(editor);
    if (text !== null) onSubmit(text);
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

MessageComposer.displayName = "MessageComposer";
