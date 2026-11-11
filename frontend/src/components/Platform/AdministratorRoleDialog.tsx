import { Dialog } from "src/components/Dialog";
import { BannerSlim, Button, Flex } from "src/components/design-system";
import { PlatformUserDto } from "src/api/platform/queries";

type AdministratorRoleDialogProps = {
  user: PlatformUserDto;
  onConfirm: () => void;
  onClose: () => void;
  isPending: boolean;
  error: Error | null;
};

export function AdministratorRoleDialog({
  user,
  onConfirm,
  onClose,
  isPending,
  error,
}: AdministratorRoleDialogProps) {
  return (
    <Dialog
      shouldOpenDialog={true}
      handleOnOpenChange={onClose}
      handleDialogClose={onClose}
      size="sm"
      title={
        user.is_platform_administrator
          ? `Remove ${user.full_name} as administrator?`
          : `Make ${user.full_name} an administrator?`
      }
      description={
        user.is_platform_administrator
          ? "They can no longer manage users, tenants and emails for this Sopho installation. Their tenant memberships stay."
          : "They can manage every user, create tenants, appoint tenant owners and see all emails on this Sopho installation."
      }
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
              label={
                user.is_platform_administrator
                  ? "Remove administrator"
                  : "Make administrator"
              }
              onClick={onConfirm}
              backgroundColor={
                user.is_platform_administrator ? "red" : "accent"
              }
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
