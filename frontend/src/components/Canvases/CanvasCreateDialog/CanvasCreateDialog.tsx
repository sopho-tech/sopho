import { useStore } from "src/store";
import { CanvasesPageState } from "src/components/Canvases/dto";
import { APP_ROUTES } from "src/constants/app_routes";
import { CanvasDto } from "src/components/Canvases/dto";
import { useNavigate } from "react-router";
import { useCreateCanvas } from "src/api/canvas/queries";
import { Form } from "src/components/design-system/Form/Form";
import {
  FormField,
  FormFieldType,
} from "src/components/design-system/Form/types";
import { SophoDialog } from "src/components/SophoDialog";
import styles from "src/components/Canvases/CanvasCreateDialog/CanvasCreateDialog.module.css";

export function CanvasCreateDialog() {
  const navigate = useNavigate();
  const canvasPageState = useStore((state) => state.canvas.canvasPageState);
  const setCanvasPageState = useStore((state) => state.canvas.setCanvasPageState);

  const handleDialogClose = () => {
    setCanvasPageState(CanvasesPageState.LIST);
  };

  const handleOnOpenChange = (open: boolean) => {
    if (!open) {
      setCanvasPageState(CanvasesPageState.LIST);
    }
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

  const formFields: FormField[] = [
    {
      key: "name",
      name: "Name",
      required: true,
      errorMessage: "Please fill name",
      type: FormFieldType.INPUT,
    },
    {
      key: "description",
      name: "Description",
      required: false,
      errorMessage: "Please enter description",
      type: FormFieldType.INPUT,
    },
  ];
  const dialogContent = (
    <Form
      fields={formFields}
      onSubmitCallback={onSubmitCallback}
      onCancelCallback={handleDialogClose}
      fieldStyleClass={styles.formFieldContainer}
      labelStyleClass={styles.formLabelContainer}
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
