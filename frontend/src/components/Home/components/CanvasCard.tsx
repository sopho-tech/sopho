import { useCallback, memo } from "react";
import { useNavigate } from "react-router";
import { Card, Flex, Text } from "src/components/design-system";
import { CanvasStats } from "./CanvasStats";
import { formatTimestamp } from "src/utils/timestamp_utils";
import { APP_ROUTES } from "src/constants/app_routes";
import type { CanvasDto } from "src/components/Canvases/dto";
import styles from "../Home.module.css";

type CanvasCardProps = {
  canvas: CanvasDto;
};

function CanvasCardComponent({ canvas }: CanvasCardProps) {
  const navigate = useNavigate();

  const handleClick = useCallback(() => {
    if (canvas.id) {
      navigate(APP_ROUTES.CANVAS.replace(":id", canvas.id));
    }
  }, [canvas.id, navigate]);

  return (
    <Card className={styles.card} onClick={canvas.id ? handleClick : undefined}>
      <Card.Header>
        <Card.Title accessbilityLevel={2}>{canvas.name}</Card.Title>
        {canvas.description && (
          <Card.Subtitle accessbilityLevel={3}>
            {canvas.description}
          </Card.Subtitle>
        )}
      </Card.Header>
      <Card.Content>
        <Flex direction="column" gap="sm">
          <Text color="subtle">{formatTimestamp(canvas.updated_at)}</Text>
          <CanvasStats canvas={canvas} />
        </Flex>
      </Card.Content>
    </Card>
  );
}

export const CanvasCard = memo(CanvasCardComponent);
