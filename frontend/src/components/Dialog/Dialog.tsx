import * as RadixDialog from "@radix-ui/react-dialog";
import DialogStyles from "src/components/Dialog/Dialog.module.css";
import { IconButton } from "src/components/design-system/IconButton/IconButton";

interface DialogProps {
  shouldOpenDialog: boolean;
  handleOnOpenChange: (open: boolean) => void;
  handleDialogClose: () => void;
  info: React.ReactElement;
  title: string;
  description?: string;
  titleAccessory?: React.ReactElement;
  dialogContentStyleClass?: string;
  modal?: boolean;
  size?: "sm" | "md";
}

export function Dialog({
  shouldOpenDialog,
  handleOnOpenChange,
  handleDialogClose,
  info,
  title,
  description,
  titleAccessory,
  dialogContentStyleClass,
  modal = true,
  size = "md",
}: DialogProps) {
  return (
    <RadixDialog.Root
      modal={modal}
      open={shouldOpenDialog}
      onOpenChange={handleOnOpenChange}
    >
      <RadixDialog.Portal>
        <RadixDialog.Overlay className={DialogStyles.dialogOverlay} />
        <RadixDialog.Content
          className={`${DialogStyles.dialogContent} ${size === "sm" ? DialogStyles.dialogContentSm : ""} ${dialogContentStyleClass || ""}`}
          onClick={(event) => event.stopPropagation()}
        >
          <div className={DialogStyles.dialogHeadersContainer}>
            <RadixDialog.Title asChild>
              <div className={DialogStyles.dialogTitle}>
                <div className={DialogStyles.dialogTitleHeading}>{title}</div>
                <div className={DialogStyles.dialogHeadersRightContainer}>
                  {titleAccessory}
                  <RadixDialog.Close asChild>
                    <IconButton
                      type="close"
                      backgroundColor="transparent"
                      iconColor="grey"
                      onClick={handleDialogClose}
                      aria-label="Close"
                    />
                  </RadixDialog.Close>
                </div>
              </div>
            </RadixDialog.Title>
            {description && (
              <RadixDialog.Description
                className={DialogStyles.dialogDescription}
              >
                {description}
              </RadixDialog.Description>
            )}
          </div>
          <div className={DialogStyles.dialogInfo}>{info}</div>
        </RadixDialog.Content>
      </RadixDialog.Portal>
    </RadixDialog.Root>
  );
}
