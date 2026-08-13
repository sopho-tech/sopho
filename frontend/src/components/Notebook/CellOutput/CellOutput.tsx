import { useCellExecutionResult } from "src/api/cell";
import {
  DataTable,
  ColumnConfig,
  Flex,
  BannerSlim,
} from "src/components/design-system";
import CellOutputStyles from "src/components/Notebook/CellOutput/CellOutput.module.css";
import CellStyles from "src/css/cell.module.css";
import { TableType } from "src/components/design-system/DataTable";
import {
  useExecutionPhase,
  ExecutionPhase,
  isAtLeast,
} from "src/components/Notebook/Cell/useExecutionPhase";
import { CellOutputSkeleton } from "src/components/Notebook/ExecutionIndicator";

interface CellOutputProps {
  cellId: string;
}

const STALE_OUTPUT_STYLE: React.CSSProperties = {
  opacity: 0.4,
  pointerEvents: "none",
  transition:
    "opacity var(--transition-duration-short) var(--transition-easing-standard-decelerate)",
};

const CURRENT_OUTPUT_STYLE: React.CSSProperties = {
  transition:
    "opacity var(--transition-duration-short) var(--transition-easing-standard-decelerate)",
};

export function CellOutput({ cellId }: CellOutputProps) {
  const { data: result } = useCellExecutionResult(cellId);
  const { isRunning, phase } = useExecutionPhase(cellId);

  if (!result && isAtLeast(phase, ExecutionPhase.VISIBLE)) {
    return (
      <Flex paddingX="lg" paddingY="lg">
        <CellOutputSkeleton />
      </Flex>
    );
  }

  if (!result) return null;

  if (result.status === "error") {
    return (
      <Flex
        alignItems="center"
        justifyContent="center"
        paddingX="lg"
        paddingY="lg"
        aria-busy={isRunning}
        sx={isRunning ? STALE_OUTPUT_STYLE : CURRENT_OUTPUT_STYLE}
      >
        <BannerSlim type="error" message={result.error.message} />
      </Flex>
    );
  }

  const { data } = result;
  if (!data) return null;

  const columns: ColumnConfig<Record<string, unknown>>[] =
    data.columns?.map((col) => ({
      key: col.column_name,
      header: col.column_name,
      type: "accessor" as const,
      accessor: col.column_name,
      dataType: col.data_type,
    })) ?? [];

  return (
    <Flex
      alignItems="center"
      justifyContent="center"
      paddingX="lg"
      paddingY="lg"
      aria-busy={isRunning}
      sx={isRunning ? STALE_OUTPUT_STYLE : CURRENT_OUTPUT_STYLE}
    >
      <DataTable
        tableType={TableType.CLIENT_SIDE_PAGINATED}
        columns={columns}
        data={data.data ?? []}
        showRowNumbers
        emptyMessage="No rows match the query"
        overallContainerStyle={`${CellStyles.outputContainerSql} ${CellOutputStyles.outputContainer}`}
        tableContainerStyle={CellOutputStyles.tableContainer}
        tableHeaderCellStyle={CellOutputStyles.tableHeaderCell}
        tableDataCellStyle={CellOutputStyles.tableDataCell}
      />
    </Flex>
  );
}
