import { useCallback, useMemo } from "react";
import { Flex } from "src/components/design-system/Flex/Flex";
import { Heading } from "../design-system";
import { useCurrentUser, useDeleteSession } from "src/api/auth_api";
import { Form } from "src/components/design-system/Form/Form";
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

  const defaultValues = useMemo(
    () =>
      user
        ? {
            id: user.id,
            username: user.username,
            email: user.email || "",
            full_name: user.full_name || "",
          }
        : null,
    [user]
  );

  const handleSubmit = useCallback(() => {}, []);
  const handleSignOut = useCallback(() => {
    deleteSessionMutation.mutate();
  }, [deleteSessionMutation]);

  return (
    <Flex
      flex="grow"
      gap="xl"
      paddingX="2xl"
      paddingY="xs"
      marginTop="xs"
      marginBottom="xs"
      marginLeft="xs"
      marginRight="xs"
      direction="column"
    >
      <Heading accessbilityLevel={1}>Profile</Heading>
      {defaultValues && (
        <Form defaultValues={defaultValues} onSubmit={handleSubmit} readonly>
          <Form.Fields className={styles.fieldsContainer}>
            <Form.Field name="id" className={styles.fieldContainer}>
              <Form.Label>User ID</Form.Label>
              <Form.Input />
            </Form.Field>
            <Form.Field name="username" className={styles.fieldContainer}>
              <Form.Label>Username</Form.Label>
              <Form.Input />
            </Form.Field>
            <Form.Field name="email" className={styles.fieldContainer}>
              <Form.Label>Email</Form.Label>
              <Form.Input />
            </Form.Field>
            <Form.Field name="full_name" className={styles.fieldContainer}>
              <Form.Label>Full Name</Form.Label>
              <Form.Input />
            </Form.Field>
          </Form.Fields>
        </Form>
      )}
      <Button
        label="Sign Out"
        backgroundColor="accent"
        size="md"
        shape="rectangle"
        leadingIconName="logout"
        onClick={handleSignOut}
        disabled={deleteSessionMutation.isPending}
      />
    </Flex>
  );
}
