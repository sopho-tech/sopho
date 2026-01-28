import { useEffect } from "react";
import { useNavigate } from "react-router";
import { Flex, Form, Heading } from "src/components/design-system";
import {
  FormField,
  FormFieldType,
} from "src/components/design-system/Form/types";
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

  const fields: FormField[] = [
    {
      key: "email",
      name: "Email",
      type: FormFieldType.INPUT,
      required: true,
      errorMessage: "Email is required",
      placeholder: "email",
      showLabel: false,
      icon: "email",
    },
    {
      key: "password",
      name: "Password",
      type: FormFieldType.INPUT_PASSWORD,
      required: true,
      errorMessage: "Password is required",
      placeholder: "password",
      showLabel: false,
      icon: "lock",
    },
  ];

  const handleSubmit = async (formData: FormData) => {
    const email = formData.get("email");
    const password = formData.get("password");

    if (typeof email !== "string" || typeof password !== "string") {
      console.error("Invalid form data");
      return;
    }

    createSessionMutation.mutate({
      email,
      password,
    });
  };

  const handleCancel = () => {
    // Cancel action - navigate back or clear form if needed
    // Currently, the form doesn't show a cancel button (showCancelButton={false})
    // This handler is kept for future use if cancel functionality is needed
  };

  return (
    <Flex
      width={"100vw"}
      height={"100vh"}
      justifyContent="center"
      alignItems="center"
      className={styles.pageContainer}
    >
      <MeshGradient
        colors={["#fcfcfd", "#fcfcfd", "#e8e8fc", "#c6bcfb", "#b5a9f4"]}
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
        <Form
          fields={fields}
          onSubmitCallback={handleSubmit}
          onCancelCallback={handleCancel}
          submitButtonText="Sign In"
          showCancelButton={false}
          showErrorBanner={false}
          fieldsContainerStyleClass={styles.fieldsContainer}
          rootStyleClass={styles.formRootContainer}
          formButtonRowStyleClass={styles.formButtonRowContainer}
          fieldStyleClass={styles.fieldContainer}
          submitOnEnter={true}
        />
      </Flex>
    </Flex>
  );
}
