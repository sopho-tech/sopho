import { useCallback, useMemo, useState } from "react";
import { CreatedByFilter } from "src/api/dto";
import { CreatedByFilterControl } from "src/components/Attribution";
import { Flex } from "src/components/design-system";
import { Table, TableType } from "src/components/Table/Table";
import { useAllCanvases } from "src/api/canvas/queries";
import {
  useCanvasesPagination,
  useCanvasActions,
} from "src/components/Canvases/hooks.tsx";
import { createCanvasesTableColumns } from "src/components/Canvases/CanvasesTable/canvasesTableColumns";
import styles from "src/components/Canvases/CanvasesTable/CanvasesTable.module.css";
import { useNavigate } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import { useTenantPath } from "src/utils/tenant";
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
  const toTenantPath = useTenantPath();
  const {
    pagination,
    handlePaginationChange,
    handleChangePageSize,
    handlePageClick,
    handleResetToFirstPage,
  } = useCanvasesPagination();
  const [createdBy, setCreatedBy] = useState<CreatedByFilter | null>(null);

  const {
    data: canvases,
    isLoading,
    isError,
  } = useAllCanvases(createdBy, pagination.pageIndex, pagination.pageSize);

  const handleCreatedByChange = useCallback(
    (value: CreatedByFilter | null) => {
      setCreatedBy(value);
      handleResetToFirstPage();
    },
    [handleResetToFirstPage],
  );

  const columns = createCanvasesTableColumns({
    onViewClick: handleViewCanvas,
    onEditClick: handleEditCanvas,
    onDeleteClick: handleDeleteCanvas,
  });

  const handleRowClick = useCallback(
    (row: CanvasDto) => {
      if (row.id) {
        void navigate(toTenantPath(APP_ROUTES.CANVAS.replace(":id", row.id)));
      }
    },
    [navigate, toTenantPath],
  );

  const tableData = useMemo(() => canvases?.data ?? [], [canvases?.data]);

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
    ],
  );

  const isEmpty =
    createdBy === null &&
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
    <Flex
      direction="column"
      flex="grow"
      marginTop="lg"
      gap="md"
      overflow="hidden"
    >
      <Flex direction="row">
        <CreatedByFilterControl
          value={createdBy}
          onValueChange={handleCreatedByChange}
        />
      </Flex>
      <Table
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
