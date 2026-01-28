import { Flex } from "src/components/design-system/Flex/Flex";
import { Heading } from "../design-system";
import { useCurrentUser, useDeleteSession } from "src/api/auth_api";
import { Form } from "src/components/design-system/Form/Form";
import { FormFieldType } from "src/components/design-system/Form/types";
import { Button } from "src/components/design-system/Button/Button";
import { useNavigate } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import styles from "./Profile.module.css";

export function Profile() {
  const { data: user } = useCurrentUser();
  const navigate = useNavigate();
  const deleteSessionMutation = useDeleteSession({
    onSuccess: () => {
      navigate(APP_ROUTES.SIGN_IN, { replace: true });
    },
    onError: (error) => {
      console.error("Sign out failed", error);
    },
  });

  const fields = [
    {
      key: "id",
      name: "User ID",
      type: FormFieldType.INPUT,
      defaultValue: user?.id || "",
    },
    {
      key: "username",
      name: "Username",
      type: FormFieldType.INPUT,
      defaultValue: user?.username || "",
    },
    {
      key: "email",
      name: "Email",
      type: FormFieldType.INPUT,
      defaultValue: user?.email || "",
    },
    {
      key: "full_name",
      name: "Full Name",
      type: FormFieldType.INPUT,
      defaultValue: user?.full_name || "",
    },
  ];

  return (
    <Flex
      flex="grow"
      gap="xl"
      paddingX="xs"
      paddingY="xs"
      marginTop="xs"
      marginBottom="xs"
      marginLeft="xs"
      marginRight="xs"
      direction="column"
    >
      <Heading accessbilityLevel={1}>Profile</Heading>
      <Form
        fields={fields}
        readonly={true}
        showSubmitButton={false}
        showCancelButton={false}
        onSubmitCallback={() => {}}
        onCancelCallback={() => {}}
        fieldsContainerStyleClass={styles.fieldsContainer}
        fieldStyleClass={styles.fieldContainer}
      />
      <Button
        label="Sign Out"
        backgroundColor="accent"
        size="md"
        shape="rectangle"
        onClick={() => {
          deleteSessionMutation.mutate();
        }}
        disabled={deleteSessionMutation.isPending}
      />
    </Flex>
  );
}
