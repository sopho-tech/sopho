import { useCallback } from "react";
import { useNavigate } from "react-router";
import { Card, Text } from "src/components/design-system";
import { CanvasStats } from "./CanvasStats";
import { formatTimestamp } from "src/utils/timestamp_utils";
import { APP_ROUTES } from "src/constants/app_routes";
import type { CanvasDto } from "src/components/Canvases/dto";

type CanvasCardProps = {
  canvas: CanvasDto;
};

export const CanvasCard = ({ canvas }: CanvasCardProps) => {
  const navigate = useNavigate();

  const handleClick = useCallback(() => {
    if (canvas.id) {
      navigate(APP_ROUTES.CANVAS.replace(":id", canvas.id));
    }
  }, [canvas.id, navigate]);

  return (
    <Card onClick={canvas.id ? handleClick : undefined}>
      <Card.Header baseAccessbilityLevel={2}>
        <Card.Title>{canvas.name}</Card.Title>
        <Card.Description>{canvas.description}</Card.Description>
      </Card.Header>
      <Card.Content>
        <Text color="subtle">{formatTimestamp(canvas.updated_at)}</Text>
        <CanvasStats canvas={canvas} />
      </Card.Content>
    </Card>
  );
};
