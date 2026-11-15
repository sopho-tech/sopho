import { UsageSummaryDto } from "src/api/ai_usage";
import { Flex, Grid, GridItem, Text } from "src/components/design-system";
import {
  GRID_GUTTER,
  TILE_COLUMN_SPAN,
} from "src/components/Settings/AIUsage/constants";
import { formatCount } from "src/components/Settings/AIUsage/format";

type UsageTileProps = {
  label: string;
  value: string;
  detail: string;
};

export function UsageTile({ label, value, detail }: UsageTileProps) {
  return (
    <Flex
      direction="column"
      gap="2xs"
      paddingX="md"
      paddingY="md"
      height="100%"
      border="default"
      borderRadius="lg"
      role="group"
      aria-label={label}
    >
      <Text fontSize="sm" color="subtle">
        {label}
      </Text>
      <Text fontSize="3xl" color="black">
        {value}
      </Text>
      <Text fontSize="sm" color="subtle">
        {detail}
      </Text>
    </Flex>
  );
}

type UsageTilesProps = {
  summary: UsageSummaryDto;
};

export function UsageTiles({ summary }: UsageTilesProps) {
  return (
    <Grid as="div" columnGutter={GRID_GUTTER} rowGutter={GRID_GUTTER}>
      <GridItem as="div" colSpan={TILE_COLUMN_SPAN}>
        <UsageTile
          label="Total tokens"
          value={formatCount(summary.total_tokens)}
          detail="Used in the time range"
        />
      </GridItem>
      <GridItem as="div" colSpan={TILE_COLUMN_SPAN}>
        <UsageTile
          label="Input tokens"
          value={formatCount(summary.input_tokens)}
          detail={`${formatCount(summary.cached_input_tokens)} read from cache`}
        />
      </GridItem>
      <GridItem as="div" colSpan={TILE_COLUMN_SPAN}>
        <UsageTile
          label="Output tokens"
          value={formatCount(summary.output_tokens)}
          detail="Written by the model"
        />
      </GridItem>
      <GridItem as="div" colSpan={TILE_COLUMN_SPAN}>
        <UsageTile
          label="Model calls"
          value={formatCount(summary.call_count)}
          detail="Requests to the AI provider"
        />
      </GridItem>
    </Grid>
  );
}
