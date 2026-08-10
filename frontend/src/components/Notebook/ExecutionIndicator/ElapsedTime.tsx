import { useEffect, useState } from "react";
import { Box, Text } from "src/components/design-system";

const TICK_MS = 100;

const ELAPSED_STYLE: React.CSSProperties = {
  fontVariantNumeric: "tabular-nums",
  minWidth: "5.5rem",
  textAlign: "right",
};

export function ElapsedTime({ startedAt }: { startedAt: number }) {
  const [elapsedMs, setElapsedMs] = useState(() => Date.now() - startedAt);

  useEffect(() => {
    setElapsedMs(Date.now() - startedAt);
    const timer = setInterval(
      () => setElapsedMs(Date.now() - startedAt),
      TICK_MS,
    );
    return () => clearInterval(timer);
  }, [startedAt]);

  return (
    <Box role="status" aria-live="polite" sx={ELAPSED_STYLE}>
      <Text color="darkGrey" fontSize="xs">
        {(elapsedMs / 1000).toFixed(1)}s
      </Text>
    </Box>
  );
}
