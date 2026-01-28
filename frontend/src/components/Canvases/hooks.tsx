import { useState } from "react";
import { useNavigate } from "react-router";
import { PaginationState, Updater } from "@tanstack/react-table";
import { APP_ROUTES } from "src/constants/app_routes";
import { useDeleteCanvas } from "src/api/canvas/queries";
import { useCanvasStore } from "src/components/Canvases/store";
import { CanvasesPageState } from "src/components/Canvases/dto";

const INITIAL_PAGE_SIZE = 20;
const INITIAL_PAGE_INDEX = 0;

export function useCanvasesPagination() {
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: INITIAL_PAGE_INDEX,
    pageSize: INITIAL_PAGE_SIZE,
  });

  const handlePaginationChange = (updaterOrValue: Updater<PaginationState>) => {
    setPagination((prev) =>
      typeof updaterOrValue === "function"
        ? updaterOrValue(prev)
        : updaterOrValue,
    );
  };

  const handleChangePageSize = (newPageSize: string) => {
    setPagination(() => ({
      pageIndex: INITIAL_PAGE_INDEX,
      pageSize: Number(newPageSize),
    }));
  };

  const handlePageClick = (newPage: number) => {
    setPagination((prev) => ({ ...prev, pageIndex: newPage }));
  };

  return {
    pagination,
    handlePaginationChange,
    handleChangePageSize,
    handlePageClick,
  };
}

export function useCanvasActions() {
  const navigate = useNavigate();
  const deleteMutation = useDeleteCanvas();
  const { setCanvasPageState } = useCanvasStore();

  const handleViewCanvas = (id: string) => {
    navigate(APP_ROUTES.CANVAS.replace(":id", id));
  };

  const handleEditCanvas = (id: string) => {
    navigate(APP_ROUTES.CANVAS.replace(":id", id));
  };

  const handleDeleteCanvas = (id: string) => {
    if (confirm("Are you sure you want to delete this canvas?")) {
      deleteMutation.mutate(id);
    }
  };

  const handleOpenCreateDialog = () => {
    setCanvasPageState(CanvasesPageState.CREATE_CANVAS_DIALOG);
  };

  return {
    handleViewCanvas,
    handleEditCanvas,
    handleDeleteCanvas,
    handleOpenCreateDialog,
  };
}
