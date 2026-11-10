import { Dialog } from "src/components/Dialog";
import { BannerSlim, Button, Flex } from "src/components/design-system";

type RemoveAiConfigurationDialogProps = {
  providerLabel: string;
  onConfirm: () => void;
  onClose: () => void;
  isPending: boolean;
  error: Error | null;
};

export function RemoveAiConfigurationDialog({
  providerLabel,
  onConfirm,
  onClose,
  isPending,
  error,
}: RemoveAiConfigurationDialogProps) {
  return (
    <Dialog
      shouldOpenDialog={true}
      handleOnOpenChange={onClose}
      handleDialogClose={onClose}
      size="sm"
      title={`Remove ${providerLabel}?`}
      description="The stored API key is deleted. AI features stop working until you add a provider again."
      info={
        <Flex direction="column" gap="md">
          {error && <BannerSlim type="error" message={error.message} />}
          <Flex gap="sm" justifyContent="flex-end">
            <Button
              type="button"
              label="Cancel"
              onClick={onClose}
              backgroundColor="white"
              size="sm"
              shape="rectangle"
              disabled={isPending}
            />
            <Button
              type="button"
              label="Remove"
              onClick={onConfirm}
              backgroundColor="red"
              size="sm"
              shape="rectangle"
              disabled={isPending}
            />
          </Flex>
        </Flex>
      }
    />
  );
}
