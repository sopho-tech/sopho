import { Dialog } from "src/components/Dialog";
import { BannerSlim, Button, Flex } from "src/components/design-system";
import { SignInProviderDto } from "src/api/auth/queries";

type DisconnectProviderDialogProps = {
  provider: SignInProviderDto;
  onConfirm: () => void;
  onClose: () => void;
  isPending: boolean;
  error: Error | null;
};

export function DisconnectProviderDialog({
  provider,
  onConfirm,
  onClose,
  isPending,
  error,
}: DisconnectProviderDialogProps) {
  return (
    <Dialog
      shouldOpenDialog={true}
      handleOnOpenChange={onClose}
      handleDialogClose={onClose}
      size="sm"
      title={`Disconnect ${provider.display_name}?`}
      description={`You will no longer be able to sign in with ${provider.display_name}. You can connect it again later from your profile.`}
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
              label="Disconnect"
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
