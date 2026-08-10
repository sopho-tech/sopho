import { Dialog } from "src/components/Dialog";
import { Button, Flex } from "src/components/design-system";

type DeleteCellDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onClose: () => void;
  onConfirm: () => void;
  isPending: boolean;
};

export const DeleteCellDialog = ({
  open,
  onOpenChange,
  onClose,
  onConfirm,
  isPending,
}: DeleteCellDialogProps) => (
  <Dialog
    shouldOpenDialog={open}
    handleOnOpenChange={onOpenChange}
    handleDialogClose={onClose}
    size="sm"
    title="Delete cell?"
    description="This removes the cell and its results. This cannot be undone."
    info={
      <Flex gap="sm" justifyContent="flex-end">
        <Button
          type="button"
          label="Cancel"
          onClick={onClose}
          backgroundColor="white"
          size="sm"
          shape="rectangle"
          disabled={isPending}
        />
        <Button
          type="button"
          label="Delete"
          onClick={onConfirm}
          backgroundColor="red"
          size="sm"
          shape="rectangle"
          disabled={isPending}
        />
      </Flex>
    }
  />
);
