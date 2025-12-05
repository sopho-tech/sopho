import { useCanvasStore } from "src/components/Canvases/store";
import { CanvasesPageState } from "src/components/Canvases/dto";
import { APP_ROUTES } from "src/constants/app_routes";
import { CanvasDto } from "src/components/Canvases/dto";
import { useNavigate } from "react-router";
import { useCreateCanvas } from "src/api/canvas/queries";
import {
  SophoForm,
  SophoFormElement,
  SophoFormElementType,
} from "src/components/SophoForm/SophoForm";
import { SophoDialog } from "src/components/SophoDialog";
import styles from "src/components/Canvases/CanvasCreateDialog/CanvasCreateDialog.module.css";

export function CanvasCreateDialog() {
  const navigate = useNavigate();
  const { canvasPageState, setCanvasPageState } = useCanvasStore();

  const handleDialogClose = () => {
    setCanvasPageState(CanvasesPageState.LIST);
  };

  const handleOnOpenChange = (open: boolean) => {
    if (!open) {
      setCanvasPageState(CanvasesPageState.LIST);
    }
    console.log("dialog state is :", open);
  };

  const mutation = useCreateCanvas({
    onSuccess: (data) => {
      setCanvasPageState(CanvasesPageState.LIST);
      if (data && data.id) {
        const canvasPath = `${APP_ROUTES.CANVASES}/${data.id}`;
        navigate(canvasPath);
      } else {
        throw Error("unexpected state");
      }
    },
    onError: (error) => {
      console.error("Error creating canvas:", error);
    },
  });

  const onSubmitCallback = (formData: FormData) => {
    const canvas: CanvasDto = {
      id: null,
      name: formData.get("name") as string,
      description: formData.get("description") as string,
      status: null,
      created_at: null,
      updated_at: null,
    };
    mutation.mutate(canvas);
  };

  const shouldOpenDialog =
    canvasPageState === CanvasesPageState.CREATE_CANVAS_DIALOG;

  const formElements: SophoFormElement[] = [
    {
      key: "name",
      name: "Name",
      required: true,
      error_message: "Please fill name",
      type: SophoFormElementType.INPUT,
    },
    {
      key: "description",
      name: "Description",
      required: false,
      error_message: "Please enter description",
      type: SophoFormElementType.INPUT,
    },
  ];
  const dialogContent = (
    <SophoForm
      formElements={formElements}
      onSubmitCallback={onSubmitCallback}
      onCancelCallback={handleDialogClose}
      formElementsStyleClass={styles.formElements}
    />
  );
  return (
    <SophoDialog
      shouldOpenDialog={shouldOpenDialog}
      handleOnOpenChange={handleOnOpenChange}
      handleDialogClose={handleDialogClose}
      info={dialogContent}
      title="New Canvas"
      description="Create a new canvas"
      dialogContentStyleClass={styles.dialogContent}
    />
  );
}
