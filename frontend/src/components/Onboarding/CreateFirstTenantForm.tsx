import { ReactNode, useCallback, useState } from "react";
import {
  BannerSlim,
  Box,
  Button,
  Flex,
  Input,
  Text,
} from "src/components/design-system";

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
  const [name, setName] = useState("");
  const canCreate = name.trim().length > 0 && !isCreating;

  const handleNameChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => setName(event.target.value),
    [],
  );
  const handleCreate = useCallback(
    () => onCreate(name.trim()),
    [onCreate, name],
  );
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
