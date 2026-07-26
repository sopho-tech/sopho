import { useCallback, useMemo } from "react";
import { Link, useNavigate } from "react-router";
import {
  Card,
  Flex,
  Badge,
  Icon,
  ToolTip,
  Text,
} from "src/components/design-system";
import type { IconType } from "src/components/design-system/datatypes";
import { APP_ROUTES } from "src/constants/app_routes";
import type { CanvasGeneratedData } from "src/components/ConversationalAnalytics/dto";

type CanvasStatProps = {
  iconType: IconType;
  count: number;
  tooltipText: string;
};

function CanvasStat({ iconType, count, tooltipText }: CanvasStatProps) {
  return (
    <ToolTip messageElement={<Text>{tooltipText}</Text>} tooltipSide="top">
      <Flex gap="xs" alignItems="center">
        <Icon type={iconType} color="grey" />
        <Badge variant="subtle">{count}</Badge>
      </Flex>
    </ToolTip>
  );
}

type CanvasGeneratedCardProps = {
  data: CanvasGeneratedData;
};

const LINK_STYLE = { color: "inherit", textDecoration: "none" };
const CARD_STYLE = { transformOrigin: "left center" };
const BADGE_STYLE = { flexShrink: 0 };
const STATS_STYLE = { flexWrap: "wrap" } as const;

export function CanvasGeneratedCard({ data }: CanvasGeneratedCardProps) {
  const navigate = useNavigate();
  const canvasPath = useMemo(
    () => APP_ROUTES.CANVAS.replace(":id", data.canvas_id),
    [data.canvas_id],
  );

  const handleClick = useCallback(() => {
    if (window.getSelection()?.toString()) return;
    navigate(canvasPath);
  }, [canvasPath, navigate]);

  const handleLinkClick = useCallback((event: React.MouseEvent) => {
    event.stopPropagation();
  }, []);

  return (
    <Card onClick={handleClick} sx={CARD_STYLE}>
      <Card.Header baseAccessbilityLevel={2}>
        <Flex
          gap="md"
          width="100%"
          alignItems="flex-start"
          justifyContent="space-between"
        >
          <Link to={canvasPath} style={LINK_STYLE} onClick={handleLinkClick}>
            <Card.Title>{data.name}</Card.Title>
          </Link>
          <Badge variant="green" shape="rounded" style={BADGE_STYLE}>
            Canvas
          </Badge>
        </Flex>
        <Card.Description>{data.description}</Card.Description>
      </Card.Header>
      <Card.Content>
        <Flex gap="lg" alignItems="center" sx={STATS_STYLE}>
          <CanvasStat
            iconType="table"
            count={data.sql_cell_count}
            tooltipText="SQL Cells"
          />
          <CanvasStat
            iconType="bar_chart"
            count={data.chart_cell_count}
            tooltipText="Chart Cells"
          />
          <CanvasStat
            iconType="layout_dashboard"
            count={data.dashboard_charts_count}
            tooltipText="Dashboard Charts"
          />
        </Flex>
      </Card.Content>
    </Card>
  );
}
