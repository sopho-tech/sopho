import { useCallback } from "react";
import { useNavigate } from "react-router";
import {
  ActionList,
  Avatar,
  Badge,
  Flex,
  Icon,
  Text,
} from "src/components/design-system";
import { TenantMembershipDto } from "src/api/tenant/queries";
import { SUSPENDED_BADGE_LABEL } from "src/components/SuspendedTenant/constants";
import { APP_ROUTES } from "src/constants/app_routes";
import { tenantPath } from "src/utils/tenant";

function TenantRow({ membership }: { membership: TenantMembershipDto }) {
  const navigate = useNavigate();
  const handleOpen = useCallback(() => {
    void navigate(tenantPath(membership.tenant_id, APP_ROUTES.INDEX));
  }, [navigate, membership.tenant_id]);

  return (
    <ActionList.Item
      leading={<Avatar name={membership.tenant_name} />}
      title={membership.tenant_name}
      trailing={
        <>
          {membership.suspended_message !== null && (
            <Badge variant="yellow">{SUSPENDED_BADGE_LABEL}</Badge>
          )}
          <Badge variant="subtle">{membership.role_display_name}</Badge>
          <Icon type="chevron_right" color="current" size="sm" />
        </>
      }
      onClick={handleOpen}
    />
  );
}

export function TenantList({
  memberships,
}: {
  memberships: TenantMembershipDto[];
}) {
  return (
    <Flex direction="column" gap="xs">
      <Text fontSize="xs" color="darkGrey">
        Your tenants
      </Text>
      <ActionList aria-label="Your tenants">
        {memberships.map((membership) => (
          <TenantRow key={membership.tenant_id} membership={membership} />
        ))}
      </ActionList>
    </Flex>
  );
}
