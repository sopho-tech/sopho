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
    () => ({
      id: user?.id || "",
      username: user?.username || "",
      email: user?.email || "",
      full_name: user?.full_name || "",
    }),
    [user?.id, user?.username, user?.email, user?.full_name]
  );

  const handleSubmit = useCallback(() => {}, []);
  const handleSignOut = useCallback(() => {
    deleteSessionMutation.mutate();
  }, [deleteSessionMutation]);

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
      <Form defaultValues={defaultValues} onSubmit={handleSubmit} readonly>
        <Form.Fields className={styles.fieldsContainer}>
          <Form.Input name="id" label="User ID" className={styles.fieldContainer} />
          <Form.Input name="username" label="Username" className={styles.fieldContainer} />
          <Form.Input name="email" label="Email" className={styles.fieldContainer} />
          <Form.Input name="full_name" label="Full Name" className={styles.fieldContainer} />
        </Form.Fields>
      </Form>
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
