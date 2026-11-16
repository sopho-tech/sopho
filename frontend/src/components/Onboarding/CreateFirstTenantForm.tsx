import { ReactNode } from "react";
import {
  BannerSlim,
  Box,
  Button,
  Flex,
  Input,
  Text,
} from "src/components/design-system";
import { useTenantNameInput } from "src/components/CreateTenantDialog/useTenantNameInput";

type CreateFirstTenantFormProps = {
  onCreate: (name: string) => void;
  isCreating: boolean;
  errorMessage: string | null;
  leadingAction: ReactNode;
};

export function CreateFirstTenantForm({
  onCreate,
  isCreating,
  errorMessage,
  leadingAction,
}: CreateFirstTenantFormProps) {
  const { name, canCreate, handleNameChange, handleCreate, handleKeyDown } =
    useTenantNameInput({ onCreate, isCreating });

  return (
    <Box as="section" aria-label="Create your tenant">
      <Flex direction="column" gap="md">
        {errorMessage && <BannerSlim type="error" message={errorMessage} />}
        <Flex direction="column" gap="xs">
          <Text>Enter tenant name</Text>
          <Input
            value={name}
            onChange={handleNameChange}
            onKeyDown={handleKeyDown}
            placeholder="Acme Analytics"
            aria-label="Enter tenant name"
            autoFocus
          />
        </Flex>
        <Flex
          direction="row"
          justifyContent="space-between"
          alignItems="center"
          gap="sm"
        >
          {leadingAction}
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
    </Box>
  );
}
