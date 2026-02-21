import { Flex, Heading, Grid, GridItem } from "src/components/design-system";
import { CanvasCard } from "./CanvasCard";
import { useIsEmptyState, useRecentlyUpdatedCanvases } from "../hooks";

const GRID_GUTTER = { base: "md", md: "lg" } as const;
const GRID_COL_SPAN = { base: 1, md: 4 } as const;

export function RecentlyUpdatedCanvases() {
  const canvasesQuery = useRecentlyUpdatedCanvases(0, 9);
  const isEmptyState = useIsEmptyState();

  if (isEmptyState || !canvasesQuery.data?.data) {
    return null;
  }

  return (
    <Flex direction="column" gap="md">
      <Heading
        accessbilityLevel={2}
        weight="medium"
        size="lg"
        textColor="subtle"
      >
        Recently Modified
      </Heading>
      <Grid as="div" columnGutter={GRID_GUTTER} rowGutter={GRID_GUTTER}>
        {canvasesQuery.data.data.map((canvas) => (
          <GridItem as="div" colSpan={GRID_COL_SPAN} key={canvas.id}>
            <CanvasCard canvas={canvas} />
          </GridItem>
        ))}
      </Grid>
    </Flex>
  );
}
