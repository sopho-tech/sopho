import { useCallback } from "react";
import { useNavigate } from "react-router";
import { Button, Flex, Form, Heading } from "src/components/design-system";
import { useCreateTenant, useMemberships } from "src/api/tenant/queries";
import { APP_ROUTES } from "src/constants/app_routes";
import { tenantPath } from "src/utils/tenant";

const DEFAULT_FORM_VALUES = { name: "" };

export function Onboarding() {
  const navigate = useNavigate();
  const memberships = useMemberships();
  const createTenant = useCreateTenant({
    onSuccess: (tenant) => {
      navigate(tenantPath(tenant.id, APP_ROUTES.INDEX), { replace: true });
    },
  });

  const handleSubmit = useCallback(
    (values: Record<string, unknown>) => {
      const name = values.name;
      if (typeof name !== "string") {
        throw new Error("Workspace name must be text");
      }
      createTenant.mutate(name.trim());
    },
    [createTenant],
  );

  const handleBack = useCallback(() => navigate(APP_ROUTES.INDEX), [navigate]);

  const hasWorkspaces = (memberships.data?.length ?? 0) > 0;

  return (
    <Flex
      width="100vw"
      height="100vh"
      justifyContent="center"
      alignItems="center"
    >
      <Flex
        direction="column"
        shadow="sm"
        borderRadius="lg"
        paddingX="xl"
        paddingY="xl"
        gap="md"
        backgroundColor="white"
      >
        <Heading accessbilityLevel={1} size="xl" textAlign="center">
          {hasWorkspaces ? "Create a workspace" : "Create your first workspace"}
        </Heading>
        <Heading
          accessbilityLevel={2}
          size="sm"
          textAlign="center"
          textColor="subtle"
        >
          Canvases, conversations and connections live inside a workspace
        </Heading>
        <Form
          defaultValues={DEFAULT_FORM_VALUES}
          onSubmit={handleSubmit}
          submitOnEnter={true}
        >
          <Form.Fields>
            <Form.Field
              name="name"
              required
              errorMessage="Workspace name is required"
            >
              <Form.Input placeholder="Workspace name" />
            </Form.Field>
          </Form.Fields>
          <Form.Actions>
            {hasWorkspaces && (
              <Button
                label="Back"
                shape="rectangle"
                size="md"
                backgroundColor="transparent"
                onClick={handleBack}
              />
            )}
            <Form.Submit label="Create workspace" />
          </Form.Actions>
        </Form>
      </Flex>
    </Flex>
  );
}
