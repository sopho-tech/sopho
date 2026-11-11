import {
  Badge,
  Box,
  Flex,
  HoverCard,
  Text,
} from "src/components/design-system";
import { PlatformUserTenantDto } from "src/api/platform/queries";
import {
  TENANT_ROLE_GRID_STYLE,
  VISIBLE_TENANT_COUNT,
} from "src/components/Platform/constants";

function TenantRoleGrid({ tenants }: { tenants: PlatformUserTenantDto[] }) {
  return (
    <Box sx={TENANT_ROLE_GRID_STYLE}>
      {tenants.map((membership) => [
        <Text key={`${membership.tenant_id}-name`} fontSize="sm" truncate>
          {membership.tenant_name}
        </Text>,
        <Badge key={`${membership.tenant_id}-role`} variant="subtle">
          {membership.role_display_name}
        </Badge>,
      ])}
    </Box>
  );
}

export function UserTenants({ tenants }: { tenants: PlatformUserTenantDto[] }) {
  if (tenants.length === 0) {
    return (
      <Text fontSize="sm" color="subtle">
        Not in any tenant
      </Text>
    );
  }
  const hiddenTenantCount = tenants.length - VISIBLE_TENANT_COUNT;

  return (
    <Flex direction="column" gap="2xs" alignItems="flex-start">
      <TenantRoleGrid tenants={tenants.slice(0, VISIBLE_TENANT_COUNT)} />
      {hiddenTenantCount > 0 && (
        <HoverCard>
          <HoverCard.Trigger>
            <Box tabIndex={0}>
              <Text fontSize="xs" color="darkGrey">
                {`+${hiddenTenantCount} more`}
              </Text>
            </Box>
          </HoverCard.Trigger>
          <HoverCard.Content align="start">
            <TenantRoleGrid tenants={tenants} />
          </HoverCard.Content>
        </HoverCard>
      )}
    </Flex>
  );
}
