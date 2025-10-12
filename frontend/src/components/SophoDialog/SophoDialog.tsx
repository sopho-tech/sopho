import * as Dialog from "@radix-ui/react-dialog";
import DialogStyles from "./SophoDialog.module.css";
import { CloseButton } from "src/components/CloseButton/CloseButton";

interface SophoDialogProps {
  shouldOpenDialog: boolean;
  handleOnOpenChange: (open: boolean) => void;
  handleDialogClose: () => void;
  triggerButton: React.ReactElement;
  content: React.ReactElement;
}

export function SophoDialog({
  shouldOpenDialog,
  handleOnOpenChange,
  handleDialogClose,
  triggerButton,
  content,
}: SophoDialogProps) {
  return (
    <Dialog.Root
      modal={true}
      open={shouldOpenDialog}
      onOpenChange={handleOnOpenChange}
    >
      <Dialog.Trigger asChild={true}>{triggerButton}</Dialog.Trigger>
      <Dialog.Portal>
        <Dialog.Overlay className={DialogStyles.dialogOverlay} />
        <Dialog.Content className={DialogStyles.dialogContent}>
          <Dialog.Title className={DialogStyles.dialogTitle}>
            <div className={DialogStyles.dialogTitleHeading}>
              Create New Notebook
            </div>
            <Dialog.Close asChild>
              <CloseButton onClick={handleDialogClose} />
            </Dialog.Close>
          </Dialog.Title>
          <Dialog.Description className={DialogStyles.dialogDescription}>
            Redirects to the new notebook after creation
          </Dialog.Description>
          {content}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
