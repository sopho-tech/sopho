import { useCallback, useState } from "react";
import {
  Badge,
  BannerSlim,
  Button,
  Flex,
  Heading,
  Table,
  Text,
} from "src/components/design-system";
import { usePlatformPermission } from "src/api/auth/queries";
import {
  PlatformTenantDto,
  usePlatformTenants,
  useRestoreTenant,
  useSuspendTenant,
} from "src/api/platform/queries";
import { CreatedTenantDto } from "src/api/tenant/queries";
import { CreateTenantDialog } from "src/components/CreateTenantDialog";
import {
  RESTORE_TENANT_LABEL,
  SUSPEND_TENANT_LABEL,
  TENANT_STATUS_BADGE_VARIANTS,
  TENANT_STATUS_NAMES,
  TENANTS_TABLE_COLUMNS,
} from "src/components/Platform/constants";
import { TenantSuspensionDialog } from "src/components/Platform/TenantSuspensionDialog";
import { PLATFORM_PERMISSIONS } from "src/constants/platform_permissions";
import { formatDate } from "src/utils/timestamp_utils";

type TenantRowProps = {
  tenant: PlatformTenantDto;
  onChangeSuspension: (tenant: PlatformTenantDto) => void;
};

function TenantRow({ tenant, onChangeSuspension }: TenantRowProps) {
  const handleChangeSuspension = useCallback(
    () => onChangeSuspension(tenant),
    [onChangeSuspension, tenant],
  );

  return (
    <Table.Row aria-label={tenant.name}>
      <Table.Cell>
        <Text fontSize="sm" truncate>
          {tenant.name}
        </Text>
      </Table.Cell>
      <Table.Cell>
        <Badge variant={TENANT_STATUS_BADGE_VARIANTS[tenant.status]}>
          {TENANT_STATUS_NAMES[tenant.status]}
        </Badge>
      </Table.Cell>
      <Table.Cell>
        {tenant.active_owner_count === 0 ? (
          <Badge variant="yellow">No active owner</Badge>
        ) : (
          <Text fontSize="sm" color="darkGrey">
            {tenant.active_owner_count}
          </Text>
        )}
      </Table.Cell>
      <Table.Cell>
        <Text fontSize="sm" color="darkGrey">
          {formatDate(tenant.created_at)}
        </Text>
      </Table.Cell>
      <Table.Cell>
        {tenant.capabilities.can_suspend && (
          <Button
            label={SUSPEND_TENANT_LABEL}
            shape="rectangle"
            size="sm"
            backgroundColor="white"
            onClick={handleChangeSuspension}
          />
        )}
        {tenant.capabilities.can_restore && (
          <Button
            label={RESTORE_TENANT_LABEL}
            shape="rectangle"
            size="sm"
            backgroundColor="white"
            onClick={handleChangeSuspension}
          />
        )}
      </Table.Cell>
    </Table.Row>
  );
}

function TenantsTable() {
  const tenants = usePlatformTenants();
  const suspendTenant = useSuspendTenant();
  const restoreTenant = useRestoreTenant();
  const [tenantAwaitingConfirmation, setTenantAwaitingConfirmation] =
    useState<PlatformTenantDto | null>(null);
  const suspensionChange =
    tenantAwaitingConfirmation?.capabilities.can_restore === true
      ? restoreTenant
      : suspendTenant;
  const { mutate: changeSuspension } = suspensionChange;
  const { reset: resetSuspendTenant } = suspendTenant;
  const { reset: resetRestoreTenant } = restoreTenant;

  const handleChangeSuspension = useCallback(
    (tenant: PlatformTenantDto) => {
      resetSuspendTenant();
      resetRestoreTenant();
      setTenantAwaitingConfirmation(tenant);
    },
    [resetSuspendTenant, resetRestoreTenant],
  );
  const handleCloseConfirmation = useCallback(
    () => setTenantAwaitingConfirmation(null),
    [],
  );
  const handleConfirmSuspensionChange = useCallback(() => {
    if (tenantAwaitingConfirmation === null) return;
    changeSuspension(tenantAwaitingConfirmation.tenant_id, {
      onSuccess: handleCloseConfirmation,
    });
  }, [changeSuspension, handleCloseConfirmation, tenantAwaitingConfirmation]);

  if (tenants.error) {
    return <BannerSlim type="error" message={tenants.error.message} />;
  }
  if (!tenants.data) {
    return null;
  }
  return (
    <>
      <Table columns={TENANTS_TABLE_COLUMNS} aria-label="Tenants">
        <Table.Header>
          <Table.HeaderCell>Tenant</Table.HeaderCell>
          <Table.HeaderCell>Status</Table.HeaderCell>
          <Table.HeaderCell>Active owners</Table.HeaderCell>
          <Table.HeaderCell>Created</Table.HeaderCell>
          <Table.HeaderCell>Actions</Table.HeaderCell>
        </Table.Header>
        {tenants.data.map((tenant) => (
          <TenantRow
            key={tenant.tenant_id}
            tenant={tenant}
            onChangeSuspension={handleChangeSuspension}
          />
        ))}
      </Table>
      {tenantAwaitingConfirmation && (
        <TenantSuspensionDialog
          tenant={tenantAwaitingConfirmation}
          onConfirm={handleConfirmSuspensionChange}
          onClose={handleCloseConfirmation}
          isPending={suspensionChange.isPending}
          error={suspensionChange.error}
        />
      )}
    </>
  );
}

export function Tenants() {
  const canListTenants = usePlatformPermission(
    PLATFORM_PERMISSIONS.TENANT_APPOINT_OWNER,
  );
  const [isCreating, setIsCreating] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);

  const handleOpenCreate = useCallback(() => {
    setNotice(null);
    setIsCreating(true);
  }, []);
  const handleCloseCreate = useCallback(() => setIsCreating(false), []);
  const handleCreated = useCallback((tenant: CreatedTenantDto) => {
    setIsCreating(false);
    setNotice(`${tenant.name} is created. You are its owner.`);
  }, []);

  return (
    <Flex direction="column" gap="lg">
      <Flex
        direction="row"
        justifyContent="space-between"
        alignItems="flex-start"
        gap="md"
      >
        <Flex direction="column" gap="2xs">
          <Heading accessibilityLevel={2} size="2xl" textColor="black">
            Global Tenants
          </Heading>
          <Text fontSize="sm" color="subtle">
            Create tenants, see who owns each one, and suspend or restore them.
          </Text>
        </Flex>
        <Button
          label="Create tenant"
          leadingIconName="add"
          shape="rectangle"
          size="sm"
          backgroundColor="accent"
          onClick={handleOpenCreate}
        />
      </Flex>
      {notice && <BannerSlim type="success" message={notice} />}
      {canListTenants && <TenantsTable />}
      {isCreating && (
        <CreateTenantDialog
          onCreated={handleCreated}
          onClose={handleCloseCreate}
        />
      )}
    </Flex>
  );
}
