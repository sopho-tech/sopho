import { Flex, Heading, Grid, GridItem } from "src/components/design-system";
import { CanvasCard } from "./CanvasCard";
import { useRecentlyUpdatedCanvases } from "../hooks";

export function RecentlyUpdatedCanvases() {
  const canvasesQuery = useRecentlyUpdatedCanvases(0, 9);

  if (!canvasesQuery.data?.data) {
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
      <Grid as="div" gutter={{ base: "md", md: "lg" }}>
        {canvasesQuery.data.data.map((canvas) => (
          <GridItem as="div" colSpan={{ base: 1, md: 4 }} key={canvas.id}>
            <CanvasCard canvas={canvas} />
          </GridItem>
        ))}
      </Grid>
    </Flex>
  );
}
