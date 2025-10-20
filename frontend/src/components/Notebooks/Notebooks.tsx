import NotebooksStyles from "src/components/Notebooks/Notebooks.module.css";
import { NotebookPageStateEnum } from "src/components/Notebooks/dto";
import { NotebookCreateDialog } from "src/components/Notebooks/NotebookCreateDialog";
import { NotebookDto } from "src/components/Notebooks/dto";
import { SophoTable, ColumnConfig } from "src/components/SophoTable/SophoTable";
import { useAllNotebooks, useDeleteNotebook } from "src/api/notebook/queries";
import { SophoTabs, type TabItem } from "src/components/SophoNavigationMenu";
import { ActionButtons } from "src/components/ActionButtons";
import { useNavigate } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import { NewAssetButton } from "src/components/NewAssetButton";
import { useNotebookStore } from "src/components/Notebooks/store";
import { formatTimestamp } from "src/utils/timestamp_utils";

export function Notebooks() {
  const { data: notebooks, isLoading, isError } = useAllNotebooks();
  const deleteMutation = useDeleteNotebook();
  const navigate = useNavigate();
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

  const columns: ColumnConfig<NotebookDto>[] = [
    {
      key: "name",
      header: "Name",
      type: "accessor",
      accessor: "name",
    },
    {
      key: "description",
      header: "Description",
      type: "accessor",
      accessor: "description",
    },
    {
      key: "created_at",
      header: "Created On",
      type: "accessor",
      accessor: "created_at",
      cell: (props) => formatTimestamp(props.getValue()),
    },
    {
      key: "updated_at",
      header: "Last Modified",
      type: "accessor",
      accessor: "updated_at",
      cell: (props) => formatTimestamp(props.getValue()),
    },
    {
      key: "actions",
      header: "Actions",
      type: "display",
      cell: (props) => (
        <ActionButtons
          connectionId={props.row.original.id}
          onViewClick={handleViewNotebook}
          onEditClick={handleEditNotebook}
          onDeleteClick={handleDeleteNotebook}
        />
      ),
    },
  ];

  const tabItems: TabItem[] = [
    {
      id: "all_notebooks",
      label: "All Notebooks",
      content: (
        <div className={NotebooksStyles.tableContainer}>
          <SophoTable
            columns={columns}
            data={notebooks ?? []}
            isLoading={isLoading}
            isError={isError}
          />
        </div>
      ),
    },
    {
      id: "my_notebooks",
      label: "My Notebooks",
      content: (
        <div>
          <p>To do</p>
        </div>
      ),
    },
    {
      id: "failing_notebooks",
      label: "Failing Notebooks",
      content: (
        <div>
          <p>To do</p>
        </div>
      ),
    },
  ];

  return (
    <div className={NotebooksStyles.notebooksContainer}>
      <div className={NotebooksStyles.notebooksHeader}>
        <h3>Notebooks</h3>
        <NewAssetButton
          buttonText="New Notebook"
          onClick={handleOpenCreateDialog}
          isLoading={isLoading}
        />
        <NotebookCreateDialog />
      </div>
      <SophoTabs items={tabItems} defaultActiveItem="all_notebooks" />
    </div>
  );
}
