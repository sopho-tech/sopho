import { Dialog } from "src/components/Dialog";
import { BannerSlim, Button, Flex } from "src/components/design-system";
import { PlatformTenantDto } from "src/api/platform/queries";
import {
  RESTORE_TENANT_DESCRIPTION,
  RESTORE_TENANT_LABEL,
  SUSPEND_TENANT_DESCRIPTION,
  SUSPEND_TENANT_LABEL,
} from "src/components/Platform/constants";

type TenantSuspensionDialogProps = {
  tenant: PlatformTenantDto;
  onConfirm: () => void;
  onClose: () => void;
  isPending: boolean;
  error: Error | null;
};

export function TenantSuspensionDialog({
  tenant,
  onConfirm,
  onClose,
  isPending,
  error,
}: TenantSuspensionDialogProps) {
  const isRestoring = tenant.capabilities.can_restore;
  const actionLabel = isRestoring ? RESTORE_TENANT_LABEL : SUSPEND_TENANT_LABEL;

  return (
    <Dialog
      shouldOpenDialog={true}
      handleOnOpenChange={onClose}
      handleDialogClose={onClose}
      size="sm"
      title={`${actionLabel} ${tenant.name}?`}
      description={
        isRestoring ? RESTORE_TENANT_DESCRIPTION : SUSPEND_TENANT_DESCRIPTION
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
              label={actionLabel}
              onClick={onConfirm}
              backgroundColor={isRestoring ? "accent" : "red"}
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
