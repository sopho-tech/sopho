import { useCallback, useMemo } from "react";
import { Flex } from "src/components/design-system";
import { SophoTable, TableType } from "src/components/SophoTable/SophoTable";
import { useAllCanvases } from "src/api/canvas/queries";
import {
  useCanvasesPagination,
  useCanvasActions,
} from "src/components/Canvases/hooks.tsx";
import { createCanvasesTableColumns } from "src/components/Canvases/CanvasesTable/canvasesTableColumns";
import styles from "src/components/Canvases/CanvasesTable/CanvasesTable.module.css";
import { useNavigate } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import { CanvasDto } from "src/components/Canvases/dto";
import { CanvasesEmptyState } from "src/components/Home/components/EmptyState";

export function CanvasesTable() {
  const {
    handleViewCanvas,
    handleEditCanvas,
    handleDeleteCanvas,
    handleOpenCreateDialog,
  } = useCanvasActions();
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
    onViewClick: handleViewCanvas,
    onEditClick: handleEditCanvas,
    onDeleteClick: handleDeleteCanvas,
  });

  const handleRowClick = useCallback(
    (row: CanvasDto) => {
      if (row.id) {
        navigate(APP_ROUTES.CANVAS.replace(":id", row.id));
      }
    },
    [navigate]
  );

  const tableData = useMemo(
    () => canvases?.data ?? [],
    [canvases?.data]
  );

  const paginationConfig = useMemo(
    () => ({
          pagination,
          totalItems: canvases?.totalItems ?? 0,
          totalPages: canvases?.totalPages ?? 0,
          currentPage: pagination.pageIndex,
          pageSize: pagination.pageSize,
          onPaginationChange: handlePaginationChange,
          onChangePageSize: handleChangePageSize,
          onPageClick: handlePageClick,
          paginationContainerClassName: styles.paginationContainer,
        }),
    [
      pagination,
      canvases?.totalItems,
      canvases?.totalPages,
      handlePaginationChange,
      handleChangePageSize,
      handlePageClick,
    ]
  );

  const isEmpty =
    !isLoading &&
    !isError &&
    (canvases?.totalItems ?? 0) === 0;

  if (isEmpty) {
    return (
      <Flex direction="column" flex="grow" marginTop="lg" overflow="hidden">
        <CanvasesEmptyState onCreateCanvas={handleOpenCreateDialog} />
      </Flex>
    );
  }

  return (
    <Flex direction="column" flex="grow" marginTop="lg" overflow="hidden">
      <SophoTable
        tableType={TableType.PAGINATED}
        columns={columns}
        data={tableData}
        isLoading={isLoading}
        isError={isError}
        onRowClick={handleRowClick}
        paginationConfig={paginationConfig}
      />
    </Flex>
  );
}
