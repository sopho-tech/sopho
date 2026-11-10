import { useCallback, useState } from "react";
import { Dialog } from "src/components/Dialog";
import {
  BannerSlim,
  Button,
  Flex,
  Input,
  Text,
} from "src/components/design-system";
import { CreatedTenantDto, useCreateTenant } from "src/api/tenant/queries";

type CreateTenantDialogProps = {
  onCreated: (tenant: CreatedTenantDto) => void;
  onClose: () => void;
};

export function CreateTenantDialog({
  onCreated,
  onClose,
}: CreateTenantDialogProps) {
  const [name, setName] = useState("");
  const createTenant = useCreateTenant({ onSuccess: onCreated });
  const { mutate: create } = createTenant;
  const canCreate = name.trim().length > 0 && !createTenant.isPending;

  const handleNameChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => setName(event.target.value),
    [],
  );
  const handleCreate = useCallback(() => create(name.trim()), [create, name]);
  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLInputElement>) => {
      if (event.key === "Enter" && canCreate) {
        event.preventDefault();
        handleCreate();
      }
    },
    [canCreate, handleCreate],
  );

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
