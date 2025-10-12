import NotebooksStyles from "src/components/Notebooks/Notebooks.module.css";
import { NotebookCreateDialog } from "src/components/Notebooks/NotebookCreateDialog";
import { NotebookDto } from "src/components/Notebooks/dto";
import { SophoTable, ColumnConfig } from "src/components/SophoTable/SophoTable";
import { useAllNotebooks, useDeleteNotebook } from "src/api/notebook/queries";
import { SophoTabs, type TabItem } from "src/components/SophoNavigationMenu";
import { ActionButtons } from "src/components/ActionButtons";
import { useNavigate } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";

export function Notebooks() {
  const { data: notebooks, isLoading, isError } = useAllNotebooks();
  const deleteMutation = useDeleteNotebook();
  const navigate = useNavigate();

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
    },
    {
      key: "updated_at",
      header: "Last Edited",
      type: "accessor",
      accessor: "updated_at",
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
        <NotebookCreateDialog />
      </div>
      <SophoTabs items={tabItems} defaultActiveItem="all_notebooks" />
    </div>
  );
}
