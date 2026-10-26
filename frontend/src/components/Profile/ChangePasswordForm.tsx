import { useCallback } from "react";
import { BannerSlim, Form } from "src/components/design-system";
import { useChangePassword } from "src/api/auth/queries";
import {
  CHANGE_PASSWORD_FORM_DEFAULT_VALUES,
  PASSWORD_CHANGED_MESSAGE,
} from "src/components/Profile/constants";
import styles from "src/components/Profile/Profile.module.css";

export function ChangePasswordForm() {
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
      change({ currentPassword, newPassword });
    },
    [change],
  );

  if (changePassword.isSuccess) {
    return <BannerSlim type="success" message={PASSWORD_CHANGED_MESSAGE} />;
  }

  return (
    <Form
      defaultValues={CHANGE_PASSWORD_FORM_DEFAULT_VALUES}
      onSubmit={handleSubmit}
    >
      <Form.Fields className={styles.fieldsContainer}>
        <Form.Field
          name="current_password"
          required
          errorMessage="Current password is required"
          className={styles.fieldContainer}
        >
          <Form.Label>Current password</Form.Label>
          <Form.Password placeholder="Current password" icon="lock" />
        </Form.Field>
        <Form.Field
          name="new_password"
          required
          errorMessage="New password is required"
          className={styles.fieldContainer}
        >
          <Form.Label>New password</Form.Label>
          <Form.Password placeholder="New password" icon="lock" />
        </Form.Field>
      </Form.Fields>
      {changePassword.error && (
        <BannerSlim type="error" message={changePassword.error.message} />
      )}
      <Form.Actions>
        <Form.Submit label="Change password" />
      </Form.Actions>
    </Form>
  );
}
