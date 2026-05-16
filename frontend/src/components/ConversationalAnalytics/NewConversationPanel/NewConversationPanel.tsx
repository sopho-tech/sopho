import { useEditor, EditorContent, EditorContext } from "@tiptap/react";
import { useMemo, useState } from "react";
import { generatePath, useNavigate } from "react-router";
import styles from "./NewConversationPanel.module.css";
import {
  Box,
  Flex,
  Heading,
  IconButton,
  Select,
} from "src/components/design-system";
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
import { useCreateConversation } from "src/api/conversational_analytics/queries";
import { useConnections } from "src/api/connection/queries";
import { useAiConfiguration } from "src/api/ai_configuration";
import { APP_ROUTES } from "src/constants/app_routes";

export function NewConversationPanel() {
  const navigate = useNavigate();
  const { mutate: createConversation, isPending } = useCreateConversation();
  const { data: connections } = useConnections();

  const [selectedConnectionId, setSelectedConnectionId] = useState("");
  const { data: aiConfiguration } = useAiConfiguration();
  const isAiReady = aiConfiguration?.status === "live";
  const aiDisabledTooltip =
    "Configure a working AI provider in Settings → AI Configurations to start new conversations.";

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
        placeholder: "Hi, how can I help you today ?",
        emptyEditorClass: "is-editor-empty",
      }),
    ],
    content: "",
    editorProps: {
      attributes: {
        class: styles.tiptap,
      },
    },
  });

  const providerValue = useMemo(() => ({ editor }), [editor]);

  const handleSubmit = () => {
    if (!isAiReady) return;
    const user_message = editor?.getText().trim() ?? "";
    if (!user_message || !editor || isPending || !selectedConnectionId) return;
    createConversation(
      { connection_id: selectedConnectionId, user_message },
      {
        onSuccess: (data) => {
          navigate(
            generatePath(
              APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.CONVERSATION,
              {
                id: data.id,
              },
            ),
          );
        },
      },
    );
  };

  return (
    <Flex direction="column" height="100%" width="100%" justifyContent="center">
      <Flex
        direction="column"
        width="100%"
        sx={{ maxWidth: "40rem", marginInline: "auto" }}
        gap="2xl"
      >
        <Heading accessbilityLevel={2} weight="normal" textAlign="center">
          Let's data
        </Heading>
        <EditorContext.Provider value={providerValue}>
          <Flex
            direction="column"
            borderRadius="2xl"
            overflow="hidden"
            sx={{ boxShadow: "var(--shadow-border-medium)" }}
          >
            <Flex
              flex="grow"
              width="100%"
              sx={{ minWidth: 0, minHeight: "70px" }}
            >
              <EditorContent editor={editor} className={styles.editorContent} />
            </Flex>
            <Flex
              justifyContent="space-between"
              alignItems="center"
              gap="sm"
              paddingTop="2xs"
              paddingBottom="sm"
              paddingX="sm"
              sx={{
                flexShrink: 0,
                backgroundColor: "var(--color-background)",
              }}
            >
              <Select
                value={selectedConnectionId}
                onValueChange={setSelectedConnectionId}
              >
                <Select.Trigger
                  placeholder="Connection"
                  className={styles.connectionSelect}
                />
                <Select.Content>
                  {(connections ?? []).map((c) => (
                    <Select.Item key={c.id} value={c.id}>
                      {c.name}
                    </Select.Item>
                  ))}
                </Select.Content>
              </Select>
              <Box sx={{ opacity: isAiReady ? 1 : 0.5 }}>
                <IconButton
                  type="arrow_up"
                  backgroundColor="accent"
                  iconColor="white"
                  tooltip={{
                    text: isAiReady ? "Send" : aiDisabledTooltip,
                    direction: "top",
                  }}
                  onClick={handleSubmit}
                  size="md"
                  disabled={!isAiReady}
                />
              </Box>
            </Flex>
          </Flex>
        </EditorContext.Provider>
      </Flex>
    </Flex>
  );
}
