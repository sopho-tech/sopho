import { ConversationDto } from "src/api/conversational_analytics";
import { Dialog } from "src/components/Dialog";
import { Form } from "src/components/design-system/Form/Form";
import styles from "src/components/ConversationalAnalytics/Sidebar/Sidebar.module.css";

type RenameConversationDialogProps = {
  target: ConversationDto;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onClose: () => void;
  onSubmit: (formData: FormData) => void;
};

export const RenameConversationDialog = ({
  target,
  open,
  onOpenChange,
  onClose,
  onSubmit,
}: RenameConversationDialogProps) => (
  <Dialog
    shouldOpenDialog={open}
    handleOnOpenChange={onOpenChange}
    handleDialogClose={onClose}
    title="Rename Conversation"
    dialogContentStyleClass={styles.dialogContent}
    info={
      <Form
        key={target.id}
        defaultValues={{ name: target.name }}
        onSubmit={onSubmit}
      >
        <Form.ErrorBanner />
        <Form.Fields>
          <Form.Field
            name="name"
            required
            errorMessage="Please enter a name"
            className={styles.formFieldContainer}
          >
            <Form.Label className={styles.formLabelContainer}>
              Enter the new conversation name
            </Form.Label>
            <Form.Input placeholder="Conversation name" />
          </Form.Field>
        </Form.Fields>
        <Form.Actions>
          <Form.Cancel onClick={onClose} />
          <Form.Submit label="Save" />
        </Form.Actions>
      </Form>
    }
  />
);
