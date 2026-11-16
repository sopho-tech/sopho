import { Dialog } from "src/components/Dialog";
import {
  BannerSlim,
  Button,
  Flex,
  Input,
  Text,
} from "src/components/design-system";
import { CreatedTenantDto, useCreateTenant } from "src/api/tenant/queries";
import { useTenantNameInput } from "src/components/CreateTenantDialog/useTenantNameInput";

type CreateTenantDialogProps = {
  onCreated: (tenant: CreatedTenantDto) => void;
  onClose: () => void;
};

export function CreateTenantDialog({
  onCreated,
  onClose,
}: CreateTenantDialogProps) {
  const createTenant = useCreateTenant({ onSuccess: onCreated });
  const { name, canCreate, handleNameChange, handleCreate, handleKeyDown } =
    useTenantNameInput({
      onCreate: createTenant.mutate,
      isCreating: createTenant.isPending,
    });

  return (
    <Dialog
      shouldOpenDialog={true}
      handleOnOpenChange={onClose}
      handleDialogClose={onClose}
      size="sm"
      title="Create a tenant"
      description="You become its owner and can invite people after it is created."
      info={
        <Flex direction="column" gap="lg">
          {createTenant.error && (
            <BannerSlim type="error" message={createTenant.error.message} />
          )}
          <Flex direction="column" gap="xs">
            <Text fontSize="sm">Name</Text>
            <Input
              value={name}
              onChange={handleNameChange}
              onKeyDown={handleKeyDown}
              placeholder="Acme Analytics"
              aria-label="Name"
              autoFocus
            />
          </Flex>
          <Flex gap="sm" justifyContent="flex-end">
            <Button
              type="button"
              label="Cancel"
              onClick={onClose}
              backgroundColor="white"
              size="sm"
              shape="rectangle"
              disabled={createTenant.isPending}
            />
            <Button
              type="button"
              label="Create tenant"
              onClick={handleCreate}
              backgroundColor="accent"
              size="sm"
              shape="rectangle"
              disabled={!canCreate}
            />
          </Flex>
        </Flex>
      }
    />
  );
}
