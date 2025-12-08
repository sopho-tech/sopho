import {
  Flex,
  IconButton,
  SegmentedControl,
} from "src/components/design-system";
import {
  useDashboardStore,
  DashboardMode,
} from "src/components/Dashboard/store";

type CanvasButtonsProps = {
  viewType: string;
  onViewTypeChange: (value: string) => void;
};

export function CanvasButtons({
  viewType,
  onViewTypeChange,
}: CanvasButtonsProps) {
  const { mode, setMode, requestSave } = useDashboardStore();
  const isEditing = mode === DashboardMode.EDITING;
  const isDashboardView = viewType === "dashboard";

  const handleEditSaveClick = () => {
    if (isEditing) {
      requestSave();
    } else {
      setMode(DashboardMode.EDITING);
    }
  };

  const handleCancelClick = () => {
    setMode(DashboardMode.VIEWING);
  };

  return (
    <Flex direction="row" gap="sm">
      <Flex direction="row" gap="2xs">
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
