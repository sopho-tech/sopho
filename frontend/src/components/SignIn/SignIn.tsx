import { useCallback, useEffect } from "react";
import { useNavigate, useSearchParams } from "react-router";
import {
  BannerSlim,
  Box,
  Button,
  Flex,
  Form,
  Heading,
  Separator,
  Text,
} from "src/components/design-system";
import { AuthPageLayout } from "src/components/AuthPageLayout";
import { authPageStyles } from "src/components/AuthPageLayout/auth_page_styles";
import { SingleSignOnButton } from "src/components/SignIn/SingleSignOnButton";
import { APP_ROUTES } from "src/constants/app_routes";
import {
  useCreateSession,
  useSessionValid,
  useSignInPageOptions,
} from "src/api/auth/queries";
import {
  SIGN_IN_ERROR_MESSAGES,
  SIGN_IN_ERROR_PARAMETER,
  SIGN_IN_FORM_DEFAULT_VALUES,
} from "src/components/SignIn/constants";
import {
  invitationPath,
  pendingInvitationToken,
} from "src/utils/pending_invitation";

function pathAfterSignIn(): string {
  const pendingToken = pendingInvitationToken();
  return pendingToken ? invitationPath(pendingToken) : APP_ROUTES.INDEX;
}

export default function SignIn() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const errorCode = searchParams.get(SIGN_IN_ERROR_PARAMETER);
  const errorMessage = errorCode
    ? SIGN_IN_ERROR_MESSAGES[errorCode]
    : undefined;
  const signInPageOptions = useSignInPageOptions();
  const { data: isAuthenticated, isLoading } = useSessionValid();
  const createSessionMutation = useCreateSession({
    onSuccess: () => {
      void navigate(pathAfterSignIn(), { replace: true });
    },
  });

  useEffect(() => {
    if (!isLoading && isAuthenticated) {
      void navigate(pathAfterSignIn(), { replace: true });
    }
  }, [isAuthenticated, isLoading, navigate]);

  const handleSubmit = useCallback(
    (values: Record<string, unknown>) => {
      const email = values.email;
      const password = values.password;

      if (typeof email !== "string" || typeof password !== "string") {
        console.error("Invalid form data");
        return;
      }

      createSessionMutation.mutate({
        email,
        password,
      });
    },
    [createSessionMutation],
  );
  const handleForgotPassword = useCallback(
    () => void navigate(APP_ROUTES.FORGOT_PASSWORD),
    [navigate],
  );

  const providers = signInPageOptions.data?.providers ?? [];

  return (
    <AuthPageLayout>
      <Flex direction="column" alignItems="center" gap="2xs">
        <Heading accessibilityLevel={1} size="xl" textAlign="center">
          Welcome back
        </Heading>
        <Text fontSize="sm" color="subtle">
          Sign in to your account
        </Text>
      </Flex>
      {errorMessage && <BannerSlim type="error" message={errorMessage} />}
      {signInPageOptions.error && (
        <BannerSlim type="error" message={signInPageOptions.error.message} />
      )}
      <Form
        defaultValues={SIGN_IN_FORM_DEFAULT_VALUES}
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
            <Form.Label className={authPageStyles.fieldLabel}>Email</Form.Label>
            <Form.Input placeholder="name@company.com" fullWidth />
          </Form.Field>
          <Form.Field
            name="password"
            required
            errorMessage="Password is required"
            className={authPageStyles.passwordFieldContainer}
          >
            <Form.Label className={authPageStyles.fieldLabel}>
              Password
            </Form.Label>
            <Form.Password placeholder="••••••••" fullWidth />
            {signInPageOptions.data?.password_reset_available && (
              <Button
                label="Forgot password?"
                shape="rectangle"
                size="sm"
                backgroundColor="ghost"
                onClick={handleForgotPassword}
              />
            )}
          </Form.Field>
        </Form.Fields>
        {createSessionMutation.error && (
          <Box role="alert">
            <BannerSlim
              type="error"
              message={createSessionMutation.error.message}
            />
          </Box>
        )}
        <Form.Actions className={authPageStyles.formButtonRowContainer}>
          <Form.Submit label="Sign in" size="md" fullWidth />
        </Form.Actions>
      </Form>
      {providers.length > 0 && (
        <Flex direction="column" gap="sm">
          <Separator label="Or continue with" />
          {providers.map((provider) => (
            <SingleSignOnButton key={provider.name} provider={provider} />
          ))}
        </Flex>
      )}
    </AuthPageLayout>
  );
}
