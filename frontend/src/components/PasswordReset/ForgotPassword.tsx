import { useCallback } from "react";
import { useNavigate } from "react-router";
import {
  BannerSlim,
  Button,
  Flex,
  Form,
  Heading,
  Text,
} from "src/components/design-system";
import { AuthPageLayout } from "src/components/AuthPageLayout";
import { authPageStyles } from "src/components/AuthPageLayout/auth_page_styles";
import { FORGOT_PASSWORD_FORM_DEFAULT_VALUES } from "src/components/PasswordReset/constants";
import { useRequestPasswordReset } from "src/api/auth/queries";
import { APP_ROUTES } from "src/constants/app_routes";

export function ForgotPassword() {
  const navigate = useNavigate();
  const requestPasswordReset = useRequestPasswordReset();
  const { mutate: requestReset } = requestPasswordReset;
  const handleSubmit = useCallback(
    (values: Record<string, unknown>) => {
      const email = values.email;
      if (typeof email !== "string") {
        throw new Error("The email field returned a value that is not text");
      }
      requestReset(email);
    },
    [requestReset],
  );
  const handleBackToSignIn = useCallback(
    () => void navigate(APP_ROUTES.SIGN_IN),
    [navigate],
  );

  return (
    <AuthPageLayout>
      <Heading accessibilityLevel={1} size="xl" textAlign="center">
        Forgot your password?
      </Heading>
      <Text fontSize="sm" color="subtle">
        Enter your email address and we'll send you a link to choose a new
        password.
      </Text>
      {requestPasswordReset.data ? (
        <BannerSlim
          type="success"
          message={requestPasswordReset.data.message}
        />
      ) : (
        <Form
          defaultValues={FORGOT_PASSWORD_FORM_DEFAULT_VALUES}
          onSubmit={handleSubmit}
          className={authPageStyles.formRootContainer}
          submitOnEnter={true}
        >
          <Form.Fields className={authPageStyles.fieldsContainer}>
            <Form.Field
              name="email"
              required
              errorMessage="Email is required"
              className={authPageStyles.fieldContainer}
            >
              <Form.Input placeholder="email" icon="email" />
            </Form.Field>
          </Form.Fields>
          {requestPasswordReset.error && (
            <BannerSlim
              type="error"
              message={requestPasswordReset.error.message}
            />
          )}
          <Form.Actions className={authPageStyles.formButtonRowContainer}>
            <Form.Submit label="Send reset link" />
          </Form.Actions>
        </Form>
      )}
      <Flex justifyContent="center">
        <Button
          label="Back to sign in"
          shape="rectangle"
          size="sm"
          backgroundColor="ghost"
          onClick={handleBackToSignIn}
        />
      </Flex>
    </AuthPageLayout>
  );
}
