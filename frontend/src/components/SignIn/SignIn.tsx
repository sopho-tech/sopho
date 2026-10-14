import { useCallback, useEffect, useMemo } from "react";
import { useNavigate, useSearchParams } from "react-router";
import {
  BannerSlim,
  Button,
  Flex,
  Form,
  Heading,
  Text,
} from "src/components/design-system";
import styles from "src/components/SignIn/SignIn.module.css";
import { MeshGradient } from "@paper-design/shaders-react";
import { APP_ROUTES } from "src/constants/app_routes";
import {
  OidcProviderDto,
  useCreateSession,
  useOidcProviders,
  useSessionValid,
} from "src/api/auth_api";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import {
  SIGN_IN_ERROR_MESSAGES,
  SIGN_IN_ERROR_PARAMETER,
} from "src/components/SignIn/constants";
import logo from "src/assets/images/logo.svg";

function SingleSignOnButton({ provider }: { provider: OidcProviderDto }) {
  const handleClick = useCallback(
    () =>
      window.location.assign(
        API_ENDPOINTS.AUTH.OIDC_START.replace(":provider_name", provider.name),
      ),
    [provider.name],
  );
  return (
    <Button
      label={`Continue with ${provider.display_name}`}
      shape="rectangle"
      size="md"
      backgroundColor="white"
      fullWidth
      onClick={handleClick}
    />
  );
}

export default function SignIn() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const errorCode = searchParams.get(SIGN_IN_ERROR_PARAMETER);
  const errorMessage = errorCode ? SIGN_IN_ERROR_MESSAGES[errorCode] : undefined;
  const providers = useOidcProviders();
  const { data: isAuthenticated, isLoading } = useSessionValid();
  const createSessionMutation = useCreateSession({
    onSuccess: () => {
      navigate(APP_ROUTES.INDEX, { replace: true });
    },
    onError: (error) => {
      console.error("Sign in failed", error);
    },
  });

  useEffect(() => {
    if (!isLoading && isAuthenticated) {
      navigate(APP_ROUTES.INDEX, { replace: true });
    }
  }, [isAuthenticated, isLoading, navigate]);

  const meshGradientColors = useMemo(
    () => ["#fcfcfd", "#fcfcfd", "#e8e8fc", "#c6bcfb", "#b5a9f4"],
    []
  );

  const defaultFormValues = { email: "", password: "" };

  const handleSubmit = useCallback(
    async (values: Record<string, unknown>) => {
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
    [createSessionMutation]
  );

  return (
    <Flex
      width={"100vw"}
      height={"100vh"}
      justifyContent="center"
      alignItems="center"
      className={styles.pageContainer}
    >
      <MeshGradient
        colors={meshGradientColors}
        distortion={0.8}
        swirl={0.1}
        grainMixer={0.47}
        grainOverlay={0.3}
        speed={1}
        className={styles.backgroundGradient}
      />
      <Flex
        direction="column"
        shadow="sm"
        borderRadius="lg"
        paddingX="xl"
        paddingY="xl"
        gap="sm"
        className={styles.formContainer}
        backgroundColor="white"
      >
        <Flex justifyContent="center" alignItems="center" marginBottom="md">
          <img src={logo} alt="Logo" className={styles.logo} />
        </Flex>
        <Heading accessbilityLevel={1} size="xl" textAlign="center">
          Welcome !
        </Heading>
        <Heading
          accessbilityLevel={2}
          size="sm"
          textAlign="center"
          textColor="subtle"
        >
          Sign in with email and password
        </Heading>
        {errorMessage && <BannerSlim type="error" message={errorMessage} />}
        {providers.error && (
          <BannerSlim type="error" message={providers.error.message} />
        )}
        <Form
          defaultValues={defaultFormValues}
          onSubmit={handleSubmit}
          className={styles.formRootContainer}
          submitOnEnter={true}
        >
          <Form.Fields className={styles.fieldsContainer}>
            <Form.Field
              name="email"
              required
              errorMessage="Email is required"
              className={styles.fieldContainer}
            >
              <Form.Input placeholder="email" icon="email" />
            </Form.Field>
            <Form.Field
              name="password"
              required
              errorMessage="Password is required"
              className={styles.fieldContainer}
            >
              <Form.Password placeholder="password" icon="lock" />
            </Form.Field>
          </Form.Fields>
          <Form.Actions className={styles.formButtonRowContainer}>
            <Form.Submit label="Sign In" />
          </Form.Actions>
        </Form>
        {providers.data && providers.data.length > 0 && (
          <Flex direction="column" gap="sm">
            <Flex justifyContent="center">
              <Text fontSize="xs" color="subtle">
                or
              </Text>
            </Flex>
            {providers.data.map((provider) => (
              <SingleSignOnButton key={provider.name} provider={provider} />
            ))}
          </Flex>
        )}
      </Flex>
    </Flex>
  );
}
