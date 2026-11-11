import { Dialog } from "src/components/Dialog";
import { BannerSlim, Button, Flex } from "src/components/design-system";
import { PlatformUserDto } from "src/api/platform/queries";

type DeactivateUserDialogProps = {
  user: PlatformUserDto;
  onConfirm: () => void;
  onClose: () => void;
  isPending: boolean;
  error: Error | null;
};

export function DeactivateUserDialog({
  user,
  onConfirm,
  onClose,
  isPending,
  error,
}: DeactivateUserDialogProps) {
  return (
    <Dialog
      shouldOpenDialog={true}
      handleOnOpenChange={onClose}
      handleDialogClose={onClose}
      size="sm"
      title={`Deactivate ${user.full_name}?`}
      description="They are signed out everywhere and can't sign in until they're reactivated. Invitations they sent are revoked. Their tenant memberships stay."
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
              label="Deactivate"
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
