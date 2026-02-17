import {
  useHandleExecuteCellPreview,
  serializeChartContent,
} from "src/components/Notebook/Cell";
import { convertValuesToFormData } from "src/components/design-system/Form/utils/values";
import { useFormCompoundContext } from "src/components/design-system/Form";
import { CellType } from "src/components/Notebook/Cell/dto";
import { ChartType } from "src/components/Chart";
import { extractChartFormData } from "src/components/Notebook/ChartCell/CellEditor/utils";
import { useCallback } from "react";
import { Button } from "src/components/design-system/Button";
import { useStore } from "src/store";

export function ChartRunButton({
  cellId,
  chartType,
}: {
  cellId: string;
  chartType: ChartType | null;
}) {
  const { form } = useFormCompoundContext();
  const handleExecuteCellPreview = useHandleExecuteCellPreview();
  const setChartContent = useStore((state) => state.cell.setChartContent);

  const handleRun = useCallback(() => {
    if (!chartType) return;
    const values = (form.state.values ?? {}) as Record<string, unknown>;
    const formData = convertValuesToFormData(values ?? {});
    const content = extractChartFormData(chartType, formData);
    const serialized = serializeChartContent(content);
    handleExecuteCellPreview(cellId, serialized, CellType.CHART);
    setChartContent(cellId, content);
  }, [cellId, chartType, form.state.values, handleExecuteCellPreview, setChartContent]);

  return (
    <Button
      key="run"
      label="Run"
      onClick={handleRun}
      backgroundColor="transparent"
      size="sm"
      shape="rectangle"
      emphasis="secondary"
    />
  );
}
