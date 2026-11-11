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
} from "src/api/platform/queries";
import { CreatedTenantDto } from "src/api/tenant/queries";
import { CreateTenantDialog } from "src/components/Platform/CreateTenantDialog";
import { TENANTS_TABLE_COLUMNS } from "src/components/Platform/constants";
import { PLATFORM_PERMISSIONS } from "src/constants/platform_permissions";
import { formatDate } from "src/utils/timestamp_utils";

function TenantRow({ tenant }: { tenant: PlatformTenantDto }) {
  return (
    <Table.Row aria-label={tenant.name}>
      <Table.Cell>
        <Text fontSize="sm" truncate>
          {tenant.name}
        </Text>
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
    </Table.Row>
  );
}

function TenantsTable() {
  const tenants = usePlatformTenants();

  if (tenants.error) {
    return <BannerSlim type="error" message={tenants.error.message} />;
  }
  if (!tenants.data) {
    return null;
  }
  return (
    <Table columns={TENANTS_TABLE_COLUMNS} aria-label="Tenants">
      <Table.Header>
        <Table.HeaderCell>Tenant</Table.HeaderCell>
        <Table.HeaderCell>Active owners</Table.HeaderCell>
        <Table.HeaderCell>Created</Table.HeaderCell>
      </Table.Header>
      {tenants.data.map((tenant) => (
        <TenantRow key={tenant.tenant_id} tenant={tenant} />
      ))}
    </Table>
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
            Create tenants and see who owns each one.
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
