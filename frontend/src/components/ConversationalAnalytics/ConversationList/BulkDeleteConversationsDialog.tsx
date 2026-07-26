import { SophoDialog } from "src/components/SophoDialog";
import { Button, Flex } from "src/components/design-system";

type BulkDeleteConversationsDialogProps = {
  count: number;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onClose: () => void;
  onConfirm: () => void;
  isPending: boolean;
};

export const BulkDeleteConversationsDialog = ({
  count,
  open,
  onOpenChange,
  onClose,
  onConfirm,
  isPending,
}: BulkDeleteConversationsDialogProps) => (
  <SophoDialog
    shouldOpenDialog={open}
    handleOnOpenChange={onOpenChange}
    handleDialogClose={onClose}
    size="sm"
    title={`Delete ${count} ${count === 1 ? "conversation" : "conversations"}?`}
    description="This also removes all of their messages and cannot be undone."
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
