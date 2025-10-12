import {
  CellOutputState,
  useCellOutputStore,
} from "src/components/Notebooks/Notebook/Cell";
import styles from "src/components/Notebooks/Notebook/ChartCell/ChartCellOutput/ChartCellOutput.module.css";
import { BarChart } from "src/components/Notebooks/Notebook/ChartCell/ChartCellOutput/BarChart";

interface ChartCellOutputProps {
  cellId: string;
}

export function ChartCellOutput({ cellId }: ChartCellOutputProps) {
  const { getOutput, getOutputState } = useCellOutputStore();
  const output = getOutput(cellId);
  const outputState = getOutputState(cellId);

  return (
    <div>
      <BarChart />
    </div>
  );
}
