import {
  Flex,
  IconButton,
  SegmentedControl,
} from "src/components/design-system";
import { useStore, DashboardMode } from "src/store";
import { useParams } from "react-router";
import { useDashboardReset } from "src/components/Dashboard/hooks";

type CanvasButtonsProps = {
  viewType: string;
  onViewTypeChange: (value: string) => void;
};

export function CanvasButtons({
  viewType,
  onViewTypeChange,
}: CanvasButtonsProps) {
  const params = useParams();
  const canvasId = params.id || "";
  const mode = useStore((state) => state.dashboard.mode);
  const isEditing = mode === DashboardMode.EDITING;
  const isDashboardView = viewType === "dashboard";
  const { handleCancelClick, handleEditSaveClick } = useDashboardReset(
    canvasId,
    isDashboardView
  );

  return (
    <Flex direction="row" gap="md">
      <Flex direction="row" gap="xs">
        <IconButton
          type="star"
          onClick={() => {}}
          backgroundColor="default"
          iconColor="grey"
        ></IconButton>
        <IconButton
          type="link"
          onClick={() => {}}
          backgroundColor="default"
          iconColor="grey"
        ></IconButton>
        {isDashboardView && (
          <IconButton
            type={isEditing ? "save" : "edit"}
            onClick={handleEditSaveClick}
            backgroundColor={isEditing ? "default" : "default"}
            iconColor={isEditing ? "accent" : "grey"}
          />
        )}
        {isDashboardView && isEditing && (
          <IconButton
            type="circle_x"
            onClick={handleCancelClick}
            backgroundColor="default"
            iconColor="red"
          />
        )}
      </Flex>
      <SegmentedControl
        options={[
          {
            value: "notebook",
            leadingIcon: "book",
            tooltip: "notebook",
          },
          {
            value: "dashboard",
            leadingIcon: "calendar",
            tooltip: "dashboard",
          },
        ]}
        value={viewType}
        onValueChange={onViewTypeChange}
        size="md"
      />
    </Flex>
  );
}
