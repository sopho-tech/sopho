import * as Dialog from "@radix-ui/react-dialog";
import DialogStyles from "./SophoDialog.module.css";
import { IconButton } from "src/components/design-system/IconButton/IconButton";

interface SophoDialogProps {
  shouldOpenDialog: boolean;
  handleOnOpenChange: (open: boolean) => void;
  handleDialogClose: () => void;
  info: React.ReactElement;
  title: string;
  description?: string;
  titleAccessory?: React.ReactElement;
  dialogContentStyleClass?: string;
}

export function SophoDialog({
  shouldOpenDialog,
  handleOnOpenChange,
  handleDialogClose,
  info,
  title,
  description,
  titleAccessory,
  dialogContentStyleClass,
}: SophoDialogProps) {
  return (
    <Dialog.Root
      modal={true}
      open={shouldOpenDialog}
      onOpenChange={handleOnOpenChange}
    >
      <Dialog.Portal>
        <Dialog.Overlay className={DialogStyles.dialogOverlay} />
        <Dialog.Content
          className={`${DialogStyles.dialogContent} ${dialogContentStyleClass || ""}`}
        >
          <div className={DialogStyles.dialogHeadersContainer}>
            <Dialog.Title asChild>
              <div className={DialogStyles.dialogTitle}>
                <div className={DialogStyles.dialogTitleHeading}>{title}</div>
                <div className={DialogStyles.dialogHeadersRightContainer}>
                  {titleAccessory}
                  <Dialog.Close asChild>
                    <IconButton
                      type="close"
                      backgroundColor="white"
                      iconColor="black"
                      onClick={handleDialogClose}
                      aria-label="Close"
                    />
                  </Dialog.Close>
                </div>
              </div>
            </Dialog.Title>
            {description && (
              <Dialog.Description className={DialogStyles.dialogDescription}>
                {description}
              </Dialog.Description>
            )}
          </div>
          <div className={DialogStyles.dialogInfo}>{info}</div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
