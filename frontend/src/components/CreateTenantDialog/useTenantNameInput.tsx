import { useCallback, useState } from "react";

type UseTenantNameInputOptions = {
  onCreate: (tenantName: string) => void;
  isCreating: boolean;
};

export function useTenantNameInput({
  onCreate,
  isCreating,
}: UseTenantNameInputOptions) {
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

  return { name, canCreate, handleNameChange, handleCreate, handleKeyDown };
}
