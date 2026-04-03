import { ConversationDto } from "src/api/conversational_analytics";
import { SophoDialog } from "src/components/SophoDialog";
import { Button, Flex, Text } from "src/components/design-system";
import styles from "src/components/ConversationalAnalytics/Sidebar/Sidebar.module.css";

type DeleteConversationDialogProps = {
  target: ConversationDto;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onClose: () => void;
  onConfirm: () => void;
  isPending: boolean;
};

export const DeleteConversationDialog = ({
  target,
  open,
  onOpenChange,
  onClose,
  onConfirm,
  isPending,
}: DeleteConversationDialogProps) => (
  <SophoDialog
    shouldOpenDialog={open}
    handleOnOpenChange={onOpenChange}
    handleDialogClose={onClose}
    title="Delete conversation"
    description="This removes the conversation and all of its messages. This cannot be undone."
    dialogContentStyleClass={styles.dialogContent}
    info={
      <Flex direction="column" gap="md">
        <Text color="darkGrey" fontSize="sm">
          Delete &quot;{target.name}&quot;?
        </Text>
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
      </Flex>
    }
  />
);
