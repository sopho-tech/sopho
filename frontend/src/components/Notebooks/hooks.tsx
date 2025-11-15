import { useState } from "react";
import { useNavigate } from "react-router";
import { PaginationState, Updater } from "@tanstack/react-table";
import { APP_ROUTES } from "src/constants/app_routes";
import { useDeleteNotebook } from "src/api/notebook/queries";
import { useNotebookStore } from "src/components/Notebooks/store";
import { NotebookPageStateEnum } from "src/components/Notebooks/dto";

const INITIAL_PAGE_SIZE = 20;
const INITIAL_PAGE_INDEX = 0;

export function useNotebooksPagination() {
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: INITIAL_PAGE_INDEX,
    pageSize: INITIAL_PAGE_SIZE,
  });

  const handlePaginationChange = (updaterOrValue: Updater<PaginationState>) => {
    setPagination((prev) =>
      typeof updaterOrValue === "function"
        ? updaterOrValue(prev)
        : updaterOrValue
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

export function useNotebookActions() {
  const navigate = useNavigate();
  const deleteMutation = useDeleteNotebook();
  const { setNotebookPageState } = useNotebookStore();

  const handleViewNotebook = (id: string) => {
    navigate(APP_ROUTES.NOTEBOOK.replace(":id", id));
  };

  const handleEditNotebook = (id: string) => {
    navigate(APP_ROUTES.NOTEBOOK.replace(":id", id));
  };

  const handleDeleteNotebook = (id: string) => {
    if (confirm("Are you sure you want to delete this notebook?")) {
      deleteMutation.mutate(id);
    }
  };

  const handleOpenCreateDialog = () => {
    setNotebookPageState(NotebookPageStateEnum.CREATE_NOTEBOOK_DIALOG);
  };

  return {
    handleViewNotebook,
    handleEditNotebook,
    handleDeleteNotebook,
    handleOpenCreateDialog,
  };
}
