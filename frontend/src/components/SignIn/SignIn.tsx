import { useCallback, useEffect } from "react";
import { useNavigate, useSearchParams } from "react-router";
import {
  BannerSlim,
  Box,
  Button,
  Flex,
  Form,
  Heading,
  Text,
} from "src/components/design-system";
import { AuthPageLayout } from "src/components/AuthPageLayout";
import { authPageStyles } from "src/components/AuthPageLayout/auth_page_styles";
import { SingleSignOnButton } from "src/components/SignIn/SingleSignOnButton";
import { APP_ROUTES } from "src/constants/app_routes";
import {
  useCreateSession,
  useSessionValid,
  useSignInOptions,
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
  const signInOptions = useSignInOptions();
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

  const providers = signInOptions.data?.providers ?? [];

  return (
    <AuthPageLayout>
      <Heading accessibilityLevel={1} size="xl" textAlign="center">
        Welcome !
      </Heading>
      <Heading
        accessibilityLevel={2}
        size="sm"
        textAlign="center"
        textColor="subtle"
      >
        Sign in with email and password
      </Heading>
      {errorMessage && <BannerSlim type="error" message={errorMessage} />}
      {signInOptions.error && (
        <BannerSlim type="error" message={signInOptions.error.message} />
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
            <Form.Input placeholder="email" icon="email" />
          </Form.Field>
          <Form.Field
            name="password"
            required
            errorMessage="Password is required"
            className={authPageStyles.fieldContainer}
          >
            <Form.Password placeholder="password" icon="lock" />
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
          <Form.Submit label="Sign In" />
        </Form.Actions>
      </Form>
      {signInOptions.data?.password_reset_available && (
        <Flex justifyContent="center">
          <Button
            label="Forgot password?"
            shape="rectangle"
            size="sm"
            backgroundColor="ghost"
            onClick={handleForgotPassword}
          />
        </Flex>
      )}
      {providers.length > 0 && (
        <Flex direction="column" gap="sm">
          <Flex justifyContent="center">
            <Text fontSize="xs" color="subtle">
              or
            </Text>
          </Flex>
          {providers.map((provider) => (
            <SingleSignOnButton key={provider.name} provider={provider} />
          ))}
        </Flex>
      )}
    </AuthPageLayout>
  );
}
