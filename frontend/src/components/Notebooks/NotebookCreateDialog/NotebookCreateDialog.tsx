import { useNotebookStore } from "src/components/Notebooks/store";
import { NotebookPageStateEnum } from "src/components/Notebooks/dto";
import { APP_ROUTES } from "src/constants/app_routes";
import { NotebookDto } from "src/components/Notebooks/dto";
import { useNavigate } from "react-router";
import { useCreateNotebook } from "src/api/notebook/queries";
import { NewAssetButton } from "src/components/NewAssetButton";
import {
  SophoForm,
  SophoFormElement,
  SophoFormElementType,
} from "src/components/SophoForm/SophoForm";
import { SophoDialog } from "src/components/SophoDialog";

export function NotebookCreateDialog() {
  const navigate = useNavigate();
  const { notebookPageState, setNotebookPageState } = useNotebookStore();

  const handleDialogClose = () => {
    setNotebookPageState(NotebookPageStateEnum.LIST);
  };

  const handleOpenCreateDialog = () => {
    setNotebookPageState(NotebookPageStateEnum.CREATE_NOTEBOOK_DIALOG);
  };

  const handleOnOpenChange = (open: boolean) => {
    if (!open) {
      setNotebookPageState(NotebookPageStateEnum.LIST);
    }
    console.log("dialog state is :", open);
  };

  const mutation = useCreateNotebook({
    onSuccess: (data) => {
      setNotebookPageState(NotebookPageStateEnum.LIST);
      console.log("Notebook created successfully:", data);
      if (data && data.id) {
        const notebookPath = `${APP_ROUTES.NOTEBOOKS}/${data.id}`;
        console.log("Navigating to notebook:", notebookPath);
        navigate(notebookPath);
      } else {
        console.error("Notebook created but ID is missing in the response");
      }
    },
    onError: (error) => {
      console.error("Error creating notebook:", error);
    },
  });

  const onSubmitCallback = (formData: FormData) => {
    console.log("Notebook submitted: ", formData);
    const notebook: NotebookDto = {
      id: null,
      name: formData.get("name") as string,
      description: formData.get("description") as string,
      status: null,
      cells: null,
      created_at: null,
      updated_at: null,
    };
    mutation.mutate(notebook);
  };

  const shouldOpenDialog =
    notebookPageState === NotebookPageStateEnum.CREATE_NOTEBOOK_DIALOG;

  const formElements: SophoFormElement[] = [
    {
      key: "name",
      name: "Name",
      required: true,
      message: "Please fill name",
      type: SophoFormElementType.INPUT,
    },
    {
      key: "description",
      name: "Description",
      required: false,
      message: "Please enter description",
      type: SophoFormElementType.INPUT,
    },
  ];

  const dialogTriggerButton = (
    <NewAssetButton
      buttonText="New Notebook"
      onClick={handleOpenCreateDialog}
      isLoading={mutation.isPending}
    />
  );
  const dialogContent = (
    <SophoForm
      formElements={formElements}
      onSubmitCallback={onSubmitCallback}
      onCancelCallback={handleDialogClose}
    />
  );
  return (
    <SophoDialog
      shouldOpenDialog={shouldOpenDialog}
      handleOnOpenChange={handleOnOpenChange}
      handleDialogClose={handleDialogClose}
      triggerButton={dialogTriggerButton}
      content={dialogContent}
    />
  );
}
