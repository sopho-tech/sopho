import { useCallback } from "react";
import { useStore } from "src/store";
import { CanvasesPageState } from "src/components/Canvases/dto";
import { APP_ROUTES } from "src/constants/app_routes";
import { CanvasDto } from "src/components/Canvases/dto";
import { useNavigate } from "react-router";
import { useCreateCanvas } from "src/api/canvas/queries";
import { Form } from "src/components/design-system/Form/Form";
import { Dialog } from "src/components/Dialog";
import styles from "src/components/Canvases/CanvasCreateDialog/CanvasCreateDialog.module.css";

const DEFAULT_FORM_VALUES = { name: "", description: "" };

export function CanvasCreateDialog() {
  const navigate = useNavigate();
  const canvasPageState = useStore((state) => state.canvas.canvasPageState);
  const setCanvasPageState = useStore(
    (state) => state.canvas.setCanvasPageState
  );

  const handleDialogClose = useCallback(() => {
    setCanvasPageState(CanvasesPageState.LIST);
  }, [setCanvasPageState]);

  const handleOnOpenChange = useCallback(
    (open: boolean) => {
      if (!open) {
        setCanvasPageState(CanvasesPageState.LIST);
      }
    },
    [setCanvasPageState]
  );

  const handleSuccess = useCallback(
    (data: CanvasDto) => {
      setCanvasPageState(CanvasesPageState.LIST);
      if (data.id) {
        const canvasPath = `${APP_ROUTES.CANVASES}/${data.id}`;
        navigate(canvasPath);
      } else {
        throw Error("unexpected state");
      }
    },
    [setCanvasPageState, navigate]
  );

  const handleError = useCallback((error: Error) => {
    console.error("Error creating canvas:", error);
  }, []);

  const mutation = useCreateCanvas({
    onSuccess: handleSuccess,
    onError: handleError,
  });

  const onSubmitCallback = useCallback(
    (formData: FormData) => {
      const canvas: Omit<CanvasDto, "id"> = {
        name: formData.get("name") as string,
        description: formData.get("description") as string,
        status: null,
        created_at: null,
        updated_at: null,
        sql_cell_count: 0,
        chart_cell_count: 0,
        dashboard_charts_count: 0,
      };
      mutation.mutate(canvas);
    },
    [mutation]
  );

  const shouldOpenDialog =
    canvasPageState === CanvasesPageState.CREATE_CANVAS_DIALOG;

  const dialogContent = (
    <Form defaultValues={DEFAULT_FORM_VALUES} onSubmit={onSubmitCallback}>
      <Form.ErrorBanner />
      <Form.Fields>
        <Form.Field
          name="name"
          required
          errorMessage="Please fill name"
          className={styles.formFieldContainer}
        >
          <Form.Label className={styles.formLabelContainer}>Name</Form.Label>
          <Form.Input placeholder="Enter name" />
        </Form.Field>
        <Form.Field
          name="description"
          errorMessage="Please enter description"
          className={styles.formFieldContainer}
        >
          <Form.Label className={styles.formLabelContainer}>Description</Form.Label>
          <Form.Input placeholder="Enter description" />
        </Form.Field>
      </Form.Fields>
      <Form.Actions>
        <Form.Cancel onClick={handleDialogClose} />
        <Form.Submit />
      </Form.Actions>
    </Form>
  );
  return (
    <Dialog
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
