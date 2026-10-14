import { useCallback } from "react";
import { useNavigate } from "react-router";
import { BannerSlim, Flex, Form, Text } from "src/components/design-system";
import { useCreateTenant } from "src/api/tenant/queries";
import { APP_ROUTES } from "src/constants/app_routes";
import { tenantPath } from "src/utils/tenant";

const DEFAULT_FORM_VALUES = { name: "" };

export function CreateWorkspaceForm() {
  const navigate = useNavigate();
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

  return (
    <Flex direction="column" gap="sm">
      <Text fontSize="sm" color="darkGrey">
        Create a workspace
      </Text>
      {createTenant.error && (
        <BannerSlim type="error" message={createTenant.error.message} />
      )}
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
          <Form.Submit label="Create workspace" />
        </Form.Actions>
      </Form>
    </Flex>
  );
}
