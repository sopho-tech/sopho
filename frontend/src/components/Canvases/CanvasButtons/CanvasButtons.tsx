import { useCallback } from "react";
import {
  Flex,
  IconButton,
  SegmentedControl,
} from "src/components/design-system";
import { IconType } from "src/components/design-system/datatypes";
import { useStore, DashboardMode } from "src/store";
import { useParams } from "react-router";
import { useAiConfiguration } from "src/api/ai_configuration";
import { useDashboardByCanvasId } from "src/api/dashboard/queries";
import {
  useAnyChartSummaryGenerating,
  useDashboardSummary,
  useGenerateAllChartSummaries,
  useGenerateDashboardSummary,
  useUpdateDashboardPrompt,
} from "src/api/ai_summary";
import { SummaryPromptControl } from "src/components/Dashboard/SummaryPrompt";
import {
  useDashboardReset,
  useRefreshDashboardCharts,
} from "src/components/Dashboard/hooks";

const SEGMENTED_OPTIONS: {
  value: string;
  leadingIcon: IconType;
  tooltip: string;
}[] = [
  { value: "notebook", leadingIcon: "table", tooltip: "notebook" },
  { value: "dashboard", leadingIcon: "layout_dashboard", tooltip: "dashboard" },
];

const SUMMARIZE_ALL_CHARTS_TOOLTIP = {
  text: "summarise all charts",
  direction: "bottom",
} as const;

const EDIT_DASHBOARD_TOOLTIP = {
  text: "edit dashboard",
  direction: "bottom",
} as const;

const REFRESH_DASHBOARD_TOOLTIP = {
  text: "refresh all charts",
  direction: "bottom",
} as const;

type CanvasButtonsProps = {
  viewType: string;
  onViewTypeChange: (value: string) => void;
  canEdit: boolean;
};

type SummarizeControlProps = {
  dashboardId: string;
  unavailableMessage: string | null;
};

function SummarizeButton({
  dashboardId,
  unavailableMessage,
}: SummarizeControlProps) {
  const { data: summary } = useDashboardSummary(dashboardId);
  const { mutate: generateSummary, isPending } =
    useGenerateDashboardSummary(dashboardId);
  const { mutate: savePrompt, isPending: isSavingPrompt } =
    useUpdateDashboardPrompt(dashboardId);
  const isGenerating = summary?.status === "GENERATING" || isPending;

  const handleClick = useCallback(() => {
    if (isGenerating) {
      return;
    }
    generateSummary();
  }, [isGenerating, generateSummary]);

  return (
    <SummaryPromptControl
      iconType="sparkles"
      actionLabel="Summarise dashboard"
      dialogTitle="Dashboard Summary Guidance Prompt"
      dialogDescription="Guide how this dashboard is described. Leave it empty to use the default style."
      userPrompt={summary?.user_prompt ?? null}
      busy={isGenerating}
      isSaving={isSavingPrompt}
      unavailableMessage={unavailableMessage}
      onGenerate={handleClick}
      onSavePrompt={savePrompt}
    />
  );
}

function SummarizeAllChartsButton({
  dashboardId,
  unavailableMessage,
}: SummarizeControlProps) {
  const { data: isAnyChartGenerating } =
    useAnyChartSummaryGenerating(dashboardId);
  const { mutate: generateAll, isPending } =
    useGenerateAllChartSummaries(dashboardId);
  const isGenerating = isAnyChartGenerating || isPending;

  const handleClick = useCallback(() => {
    if (isGenerating) {
      return;
    }
    generateAll();
  }, [isGenerating, generateAll]);

  return (
    <IconButton
      type="wand_sparkles"
      onClick={handleClick}
      busy={isGenerating}
      disabledReason={unavailableMessage ?? undefined}
      backgroundColor="default"
      iconColor="grey"
      aria-label="Summarise all charts"
      tooltip={SUMMARIZE_ALL_CHARTS_TOOLTIP}
    />
  );
}

function RefreshAllChartsButton() {
  const { refreshAll, isRefreshing } = useRefreshDashboardCharts();

  return (
    <IconButton
      type="refresh"
      onClick={refreshAll}
      busy={isRefreshing}
      busyAnimation="spin"
      backgroundColor="default"
      iconColor="grey"
      tooltip={REFRESH_DASHBOARD_TOOLTIP}
    />
  );
}

export function CanvasButtons({
  viewType,
  onViewTypeChange,
  canEdit,
}: CanvasButtonsProps) {
  const params = useParams();
  const canvasId = params.id || "";
  const mode = useStore((state) => state.dashboard.mode);
  const isEditing = mode === DashboardMode.EDITING;
  const isDashboardView = viewType === "dashboard";
  const dashboardQuery = useDashboardByCanvasId(canvasId);
  const { data: aiConfiguration } = useAiConfiguration();
  const { handleCancelClick, handleEditSaveClick } = useDashboardReset(
    canvasId,
    isDashboardView,
  );

  const dashboardId = dashboardQuery.data?.id;
  const hasSavedCharts = (dashboardQuery.data?.layout?.length ?? 0) > 0;
  const isAiAvailable = aiConfiguration?.is_ai_available === true;
  const aiUnavailableMessage = isAiAvailable
    ? null
    : (aiConfiguration?.unavailable_message ?? null);
  const showSummarizeControls =
    canEdit &&
    isDashboardView &&
    !isEditing &&
    hasSavedCharts &&
    (isAiAvailable || aiUnavailableMessage !== null) &&
    !!dashboardId;

  return (
    <Flex direction="row" gap="md">
      <Flex direction="row" gap="md">
        {canEdit && isDashboardView && (
          <IconButton
            type={isEditing ? "save" : "edit"}
            onClick={handleEditSaveClick}
            backgroundColor={isEditing ? "default" : "default"}
            iconColor={isEditing ? "accent" : "grey"}
            tooltip={EDIT_DASHBOARD_TOOLTIP}
          />
        )}
        {canEdit && isDashboardView && isEditing && (
          <IconButton
            type="circle_x"
            onClick={handleCancelClick}
            backgroundColor="default"
            iconColor="red"
          />
        )}
        {isDashboardView && hasSavedCharts && <RefreshAllChartsButton />}
        {showSummarizeControls && (
          <SummarizeAllChartsButton
            dashboardId={dashboardId}
            unavailableMessage={aiUnavailableMessage}
          />
        )}
        {showSummarizeControls && (
          <SummarizeButton
            dashboardId={dashboardId}
            unavailableMessage={aiUnavailableMessage}
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
