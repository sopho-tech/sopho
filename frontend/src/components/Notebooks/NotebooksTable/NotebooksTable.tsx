import { SophoTable, TableType } from "src/components/SophoTable/SophoTable";
import { useAllNotebooks } from "src/api/notebook/queries";
import { useNotebooksPagination } from "src/components/Notebooks/hooks.tsx";
import { createNotebooksTableColumns } from "src/components/Notebooks/NotebooksTable/notebooksTableColumns";
import { Flex } from "src/components/design-system/Flex/Flex";
import NotebooksTableStyles from "src/components/Notebooks/NotebooksTable/NotebooksTable.module.css";

type NotebooksTableProps = {
  onViewClick: (id: string) => void;
  onEditClick: (id: string) => void;
  onDeleteClick: (id: string) => void;
};

export function NotebooksTable({
  onViewClick,
  onEditClick,
  onDeleteClick,
}: NotebooksTableProps) {
  const {
    pagination,
    handlePaginationChange,
    handleChangePageSize,
    handlePageClick,
  } = useNotebooksPagination();

  const {
    data: notebooks,
    isLoading,
    isError,
  } = useAllNotebooks(pagination.pageIndex, pagination.pageSize);

  const columns = createNotebooksTableColumns({
    onViewClick,
    onEditClick,
    onDeleteClick,
  });

  return (
    <Flex direction="column" flex="grow" marginTop="lg" overflow="hidden">
      <SophoTable
        tableType={TableType.PAGINATED}
        columns={columns}
        data={notebooks?.data ?? []}
        isLoading={isLoading}
        isError={isError}
        paginationConfig={{
          pagination,
          totalItems: notebooks?.totalItems ?? 0,
          totalPages: notebooks?.totalPages ?? 0,
          currentPage: pagination.pageIndex,
          pageSize: pagination.pageSize,
          onPaginationChange: handlePaginationChange,
          onChangePageSize: handleChangePageSize,
          onPageClick: handlePageClick,
          paginationContainerClassName:
            NotebooksTableStyles.paginationContainer,
        }}
        tableContainerStyle={NotebooksTableStyles.tableContainer}
        tableHeaderCellStyle={NotebooksTableStyles.tableHeaderCell}
        tableDataCellStyle={NotebooksTableStyles.tableDataCell}
        tableFirstHeaderCellStyle={NotebooksTableStyles.tableFirstHeaderCell}
        tableLastHeaderCellStyle={NotebooksTableStyles.tableLastHeaderCell}
      />
    </Flex>
  );
}
