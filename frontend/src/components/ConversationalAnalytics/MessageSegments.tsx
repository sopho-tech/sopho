import { Fragment } from "react";
import { Badge } from "src/components/design-system";
import type { MessageSegment } from "src/components/ConversationalAnalytics/dto";

function isMessageSegment(value: unknown): value is MessageSegment {
  if (typeof value !== "object" || value === null) return false;
  const segment = value as Record<string, unknown>;
  if (segment.type === "TEXT") return typeof segment.text === "string";
  if (segment.type === "COMMAND") return typeof segment.name === "string";
  return false;
}

export function parseMessageSegments(content: string): MessageSegment[] | null {
  try {
    const parsed = JSON.parse(content);
    if (Array.isArray(parsed) && parsed.every(isMessageSegment)) {
      return parsed as MessageSegment[];
    }
    return null;
  } catch {
    return null;
  }
}

export function segmentsToText(segments: MessageSegment[]): string {
  return segments
    .map((segment) =>
      segment.type === "COMMAND" ? `/${segment.name}` : segment.text,
    )
    .join("");
}

type MessageSegmentsProps = {
  segments: MessageSegment[];
};

export function MessageSegments({ segments }: MessageSegmentsProps) {
  return (
    <>
      {segments.map((segment, i) =>
        segment.type === "COMMAND" ? (
          <Badge
            key={`command-${i}`}
            variant="command"
            shape="rounded"
            size="md"
          >{`/${segment.name}`}</Badge>
        ) : (
          <Fragment key={`text-${i}`}>{segment.text}</Fragment>
        ),
      )}
    </>
  );
}
