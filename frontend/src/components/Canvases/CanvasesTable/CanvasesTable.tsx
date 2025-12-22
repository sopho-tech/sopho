import { Flex } from "src/components/design-system";
import { SophoTable, TableType } from "src/components/SophoTable/SophoTable";
import { useAllCanvases } from "src/api/canvas/queries";
import { useCanvasesPagination } from "src/components/Canvases/hooks.tsx";
import { createCanvasesTableColumns } from "src/components/Canvases/CanvasesTable/canvasesTableColumns";
import styles from "src/components/Canvases/CanvasesTable/CanvasesTable.module.css";
import { useNavigate } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import { CanvasDto } from "src/components/Canvases/dto";

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
  const navigate = useNavigate();
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

  const handleRowClick = (row: CanvasDto) => {
    if (row.id) {
      navigate(APP_ROUTES.CANVAS.replace(":id", row.id));
    }
  };

  return (
    <Flex direction="column" flex="grow" marginTop="lg" overflow="hidden">
      <SophoTable
        tableType={TableType.PAGINATED}
        columns={columns}
        data={canvases?.data ?? []}
        isLoading={isLoading}
        isError={isError}
        onRowClick={handleRowClick}
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
      />
    </Flex>
  );
}
