import { NodeViewWrapper, type NodeViewProps } from "@tiptap/react";
import { Badge } from "src/components/design-system";

export function SlashCommandBadge({ node }: NodeViewProps) {
  const commandName = node.attrs.commandName as string;
  return (
    <NodeViewWrapper as="span" contentEditable={false}>
      <Badge
        variant="command"
        shape="rounded"
        size="sm"
      >{`/${commandName}`}</Badge>
    </NodeViewWrapper>
  );
}
