import { memo } from "react";
import { Flex } from "src/components/design-system";
import { CanvasStatItem } from "./CanvasStatItem";
import type { CanvasDto } from "src/components/Canvases/dto";

const FLEX_SX = { flexWrap: "wrap" } as const;

type CanvasStatsProps = {
  canvas: CanvasDto;
};

function CanvasStatsComponent({ canvas }: CanvasStatsProps) {
  return (
    <Flex gap="lg" alignItems="center" sx={FLEX_SX}>
      <CanvasStatItem
        iconType="table"
        count={canvas.sql_cell_count}
        tooltipText="SQL Cells"
      />
      <CanvasStatItem
        iconType="bar_chart"
        count={canvas.chart_cell_count}
        tooltipText="Chart Cells"
      />
      <CanvasStatItem
        iconType="layout_dashboard"
        count={canvas.dashboard_charts_count}
        tooltipText="Dashboard Charts"
      />
    </Flex>
  );
}

export const CanvasStats = memo(CanvasStatsComponent);
