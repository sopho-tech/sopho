import { Flex } from "src/components/design-system";
import { SophoTable, TableType } from "src/components/SophoTable/SophoTable";
import { useAllCanvases } from "src/api/canvas/queries";
import { useCanvasesPagination } from "src/components/Canvases/hooks.tsx";
import { createCanvasesTableColumns } from "src/components/Canvases/CanvasesTable/canvasesTableColumns";
import styles from "src/components/Canvases/CanvasesTable/CanvasesTable.module.css";

type CanvasesTableProps = {
  onViewClick: (id: string) => void;
  onEditClick: (id: string) => void;
  onDeleteClick: (id: string) => void;
};

export function CanvasesTable({
  onViewClick,
  onEditClick,
  onDeleteClick,
}: CanvasesTableProps) {
  const {
    pagination,
    handlePaginationChange,
    handleChangePageSize,
    handlePageClick,
  } = useCanvasesPagination();

  const {
    data: canvases,
    isLoading,
    isError,
  } = useAllCanvases(pagination.pageIndex, pagination.pageSize);

  const columns = createCanvasesTableColumns({
    onViewClick,
    onEditClick,
    onDeleteClick,
  });

  return (
    <Flex direction="column" flex="grow" marginTop="lg" overflow="hidden">
      <SophoTable
        tableType={TableType.PAGINATED}
        columns={columns}
        data={canvases?.data ?? []}
        isLoading={isLoading}
        isError={isError}
        paginationConfig={{
          pagination,
          totalItems: canvases?.totalItems ?? 0,
          totalPages: canvases?.totalPages ?? 0,
          currentPage: pagination.pageIndex,
          pageSize: pagination.pageSize,
          onPaginationChange: handlePaginationChange,
          onChangePageSize: handleChangePageSize,
          onPageClick: handlePageClick,
          paginationContainerClassName: styles.paginationContainer,
        }}
        tableContainerStyle={styles.tableContainer}
        tableHeaderCellStyle={styles.tableHeaderCell}
        tableDataCellStyle={styles.tableDataCell}
        tableFirstHeaderCellStyle={styles.tableFirstHeaderCell}
        tableLastHeaderCellStyle={styles.tableLastHeaderCell}
      />
    </Flex>
  );
}
