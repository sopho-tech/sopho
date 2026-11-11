import { useCallback } from "react";
import { Dialog } from "src/components/Dialog";
import { BannerSlim, Form } from "src/components/design-system";
import { useChangePassword } from "src/api/auth/queries";
import { CHANGE_PASSWORD_FORM_DEFAULT_VALUES } from "src/components/Profile/constants";
import styles from "src/components/Profile/Profile.module.css";

type ChangePasswordDialogProps = {
  onChanged: () => void;
  onClose: () => void;
};

export function ChangePasswordDialog({
  onChanged,
  onClose,
}: ChangePasswordDialogProps) {
  const changePassword = useChangePassword();
  const { mutate: change } = changePassword;
  const handleSubmit = useCallback(
    (values: Record<string, unknown>) => {
      const currentPassword = values.current_password;
      const newPassword = values.new_password;
      if (
        typeof currentPassword !== "string" ||
        typeof newPassword !== "string"
      ) {
        throw new Error("The password form returned values that are not text");
      }
      change({ currentPassword, newPassword }, { onSuccess: onChanged });
    },
    [change, onChanged],
  );

  return (
    <Dialog
      shouldOpenDialog={true}
      handleOnOpenChange={onClose}
      handleDialogClose={onClose}
      size="sm"
      title="Change password"
      description="Your other sessions are signed out after the change."
      info={
        <Form
          defaultValues={CHANGE_PASSWORD_FORM_DEFAULT_VALUES}
          onSubmit={handleSubmit}
          submitOnEnter={true}
        >
          {changePassword.error && (
            <BannerSlim type="error" message={changePassword.error.message} />
          )}
          <Form.Fields className={styles.fieldsContainer}>
            <Form.Field
              name="current_password"
              required
              errorMessage="Current password is required"
              className={styles.fieldContainer}
            >
              <Form.Label>Current password</Form.Label>
              <Form.Password placeholder="Current password" fullWidth />
            </Form.Field>
            <Form.Field
              name="new_password"
              required
              errorMessage="New password is required"
              className={styles.fieldContainer}
            >
              <Form.Label>New password</Form.Label>
              <Form.Password placeholder="New password" fullWidth />
            </Form.Field>
          </Form.Fields>
          <Form.Actions>
            <Form.Cancel onClick={onClose} />
            <Form.Submit label="Change password" />
          </Form.Actions>
        </Form>
      }
    />
  );
}
