import { useCallback, useState } from "react";
import { Dialog } from "src/components/Dialog";
import { BannerSlim, Button, Flex, Select } from "src/components/design-system";
import {
  PlatformTenantDto,
  PlatformUserDto,
  useAppointOwner,
  usePlatformTenants,
} from "src/api/platform/queries";

function activeOwnersLabel(tenant: PlatformTenantDto): string {
  if (tenant.active_owner_count === 0) {
    return "no active owner";
  }
  return tenant.active_owner_count === 1
    ? "1 active owner"
    : `${tenant.active_owner_count} active owners`;
}

type AppointOwnerDialogProps = {
  user: PlatformUserDto;
  onAppointed: (tenant: PlatformTenantDto) => void;
  onClose: () => void;
};

export function AppointOwnerDialog({
  user,
  onAppointed,
  onClose,
}: AppointOwnerDialogProps) {
  const tenants = usePlatformTenants();
  const appointOwner = useAppointOwner();
  const { mutate: appoint } = appointOwner;
  const [selectedTenantId, setSelectedTenantId] = useState("");

  const handleAppoint = useCallback(() => {
    const selectedTenant = tenants.data?.find(
      (tenant) => tenant.tenant_id === selectedTenantId,
    );
    if (!selectedTenant) {
      throw new Error("No tenant is selected");
    }
    appoint(
      { tenantId: selectedTenant.tenant_id, userId: user.id },
      { onSuccess: () => onAppointed(selectedTenant) },
    );
  }, [appoint, onAppointed, selectedTenantId, tenants.data, user.id]);

  return (
    <Dialog
      shouldOpenDialog={true}
      handleOnOpenChange={onClose}
      handleDialogClose={onClose}
      size="sm"
      title={`Make ${user.full_name} an owner`}
      description="Owners manage a tenant's members, roles and settings. Pick the tenant."
      info={
        <Flex direction="column" gap="md">
          {tenants.error && (
            <BannerSlim type="error" message={tenants.error.message} />
          )}
          {appointOwner.error && (
            <BannerSlim type="error" message={appointOwner.error.message} />
          )}
          {tenants.data && (
            <Select
              value={selectedTenantId}
              onValueChange={setSelectedTenantId}
            >
              <Select.Trigger
                placeholder="Choose a tenant"
                aria-label="Tenant"
              />
              <Select.Content>
                {tenants.data.map((tenant) => (
                  <Select.Item key={tenant.tenant_id} value={tenant.tenant_id}>
                    {`${tenant.name} · ${activeOwnersLabel(tenant)}`}
                  </Select.Item>
                ))}
              </Select.Content>
            </Select>
          )}
          <Flex gap="sm" justifyContent="flex-end">
            <Button
              type="button"
              label="Cancel"
              onClick={onClose}
              backgroundColor="white"
              size="sm"
              shape="rectangle"
              disabled={appointOwner.isPending}
            />
            <Button
              type="button"
              label="Make owner"
              onClick={handleAppoint}
              backgroundColor="accent"
              size="sm"
              shape="rectangle"
              disabled={selectedTenantId === "" || appointOwner.isPending}
            />
          </Flex>
        </Flex>
      }
    />
  );
}
