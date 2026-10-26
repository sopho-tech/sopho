import { useCallback } from "react";
import { useLocation, useNavigate } from "react-router";
import {
  BannerSlim,
  Button,
  Flex,
  Form,
  Heading,
} from "src/components/design-system";
import { AuthPageLayout } from "src/components/AuthPageLayout";
import { authPageStyles } from "src/components/AuthPageLayout/auth_page_styles";
import {
  INCOMPLETE_RESET_LINK_MESSAGE,
  INVALID_RESET_LINK_ERROR_CODE,
  PASSWORD_UPDATED_MESSAGE,
  RESET_PASSWORD_FORM_DEFAULT_VALUES,
} from "src/components/PasswordReset/constants";
import { useResetPassword } from "src/api/auth/queries";
import { ApiError } from "src/api/dto";
import { APP_ROUTES } from "src/constants/app_routes";
import { linkTokenFromHash } from "src/utils/pending_invitation";

function ResetPasswordForm({ token }: { token: string }) {
  const navigate = useNavigate();
  const resetPassword = useResetPassword();
  const { mutate: reset } = resetPassword;
  const handleSubmit = useCallback(
    (values: Record<string, unknown>) => {
      const newPassword = values.new_password;
      if (typeof newPassword !== "string") {
        throw new Error("The password field returned a value that is not text");
      }
      reset({ token, newPassword });
    },
    [reset, token],
  );
  const handleSignIn = useCallback(
    () => void navigate(APP_ROUTES.SIGN_IN, { replace: true }),
    [navigate],
  );
  const handleRequestNewLink = useCallback(
    () => void navigate(APP_ROUTES.FORGOT_PASSWORD, { replace: true }),
    [navigate],
  );
  const linkIsInvalid =
    resetPassword.error instanceof ApiError &&
    resetPassword.error.errorCode === INVALID_RESET_LINK_ERROR_CODE;

  if (resetPassword.isSuccess) {
    return (
      <Flex direction="column" gap="md">
        <BannerSlim type="success" message={PASSWORD_UPDATED_MESSAGE} />
        <Button
          label="Sign in"
          shape="rectangle"
          size="md"
          backgroundColor="accent"
          fullWidth
          onClick={handleSignIn}
        />
      </Flex>
    );
  }

  return (
    <Form
      defaultValues={RESET_PASSWORD_FORM_DEFAULT_VALUES}
      onSubmit={handleSubmit}
      className={authPageStyles.formRootContainer}
      submitOnEnter={true}
    >
      <Form.Fields className={authPageStyles.fieldsContainer}>
        <Form.Field
          name="new_password"
          required
          errorMessage="Password is required"
          className={authPageStyles.fieldContainer}
        >
          <Form.Password placeholder="New password" icon="lock" />
        </Form.Field>
      </Form.Fields>
      {resetPassword.error && (
        <BannerSlim type="error" message={resetPassword.error.message} />
      )}
      <Form.Actions className={authPageStyles.formButtonRowContainer}>
        {linkIsInvalid ? (
          <Button
            label="Request a new link"
            shape="rectangle"
            size="md"
            backgroundColor="accent"
            onClick={handleRequestNewLink}
          />
        ) : (
          <Form.Submit label="Choose new password" />
        )}
      </Form.Actions>
    </Form>
  );
}

export function ResetPassword() {
  const location = useLocation();
  const token = linkTokenFromHash(location.hash);

  return (
    <AuthPageLayout>
      <Heading accessibilityLevel={1} size="xl" textAlign="center">
        Choose a new password
      </Heading>
      {token ? (
        <ResetPasswordForm token={token} />
      ) : (
        <BannerSlim type="error" message={INCOMPLETE_RESET_LINK_MESSAGE} />
      )}
    </AuthPageLayout>
  );
}
