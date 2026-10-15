import { useCallback, useEffect, useMemo } from "react";
import { useNavigate } from "react-router";
import {
  BannerSlim,
  Box,
  Flex,
  Form,
  Heading,
} from "src/components/design-system";
import styles from "src/components/SignIn/SignIn.module.css";
import { MeshGradient } from "@paper-design/shaders-react";
import { APP_ROUTES } from "src/constants/app_routes";
import { useCreateSession, useSessionValid } from "src/api/auth_api";
import logo from "src/assets/images/logo.svg";

export default function SignIn() {
  const navigate = useNavigate();
  const { data: isAuthenticated, isLoading } = useSessionValid();
  const createSessionMutation = useCreateSession({
    onSuccess: () => {
      void navigate(APP_ROUTES.INDEX, { replace: true });
    },
  });

  useEffect(() => {
    if (!isLoading && isAuthenticated) {
      void navigate(APP_ROUTES.INDEX, { replace: true });
    }
  }, [isAuthenticated, isLoading, navigate]);

  const meshGradientColors = useMemo(
    () => ["#fcfcfd", "#fcfcfd", "#e8e8fc", "#c6bcfb", "#b5a9f4"],
    [],
  );

  const defaultFormValues = { email: "", password: "" };

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
          {createSessionMutation.error && (
            <Box role="alert">
              <BannerSlim
                type="error"
                message={createSessionMutation.error.message}
              />
            </Box>
          )}
          <Form.Actions className={styles.formButtonRowContainer}>
            <Form.Submit label="Sign In" />
          </Form.Actions>
        </Form>
      </Flex>
    </Flex>
  );
}
