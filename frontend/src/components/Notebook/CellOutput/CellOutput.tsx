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

interface CellOutputProps {
  cellId: string;
}

export function CellOutput({ cellId }: CellOutputProps) {
  const { data: result } = useCellExecutionResult(cellId);

  if (!result) return null;

  if (result.status === "error") {
    return (
      <Flex
        alignItems="center"
        justifyContent="center"
        paddingX="lg"
        paddingY="lg"
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
    >
      <DataTable
        tableType={TableType.CLIENT_SIDE_PAGINATED}
        columns={columns}
        data={data.data ?? []}
        emptyMessage="No rows match the query"
        overallContainerStyle={`${CellStyles.outputContainerSql} ${CellOutputStyles.outputContainer}`}
        tableContainerStyle={CellOutputStyles.tableContainer}
        tableHeaderCellStyle={CellOutputStyles.tableHeaderCell}
        tableDataCellStyle={CellOutputStyles.tableDataCell}
      />
    </Flex>
  );
}
