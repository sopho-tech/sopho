import {
  Flex,
  IconButton,
  SegmentedControl,
} from "src/components/design-system";
import { IconType } from "src/components/design-system/datatypes";
import { useStore, DashboardMode } from "src/store";
import { useParams } from "react-router";
import { useDashboardReset } from "src/components/Dashboard/hooks";

const SEGMENTED_OPTIONS: {
  value: string;
  leadingIcon: IconType;
  tooltip: string;
}[] = [
  { value: "notebook", leadingIcon: "book", tooltip: "notebook" },
  { value: "dashboard", leadingIcon: "calendar", tooltip: "dashboard" },
];

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
        options={SEGMENTED_OPTIONS}
        value={viewType}
        onValueChange={onViewTypeChange}
        size="md"
      />
    </Flex>
  );
}
