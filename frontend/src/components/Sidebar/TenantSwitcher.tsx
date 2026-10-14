import { useCallback } from "react";
import { useNavigate } from "react-router";
import SidebarStyles from "src/components/Sidebar/Sidebar.module.css";
import { DropdownMenu } from "src/components/design-system";
import {
  TenantMembershipDto,
  useCurrentMembership,
  useMemberships,
} from "src/api/tenant/queries";
import { APP_ROUTES } from "src/constants/app_routes";
import { tenantPath, useTenantId } from "src/utils/tenant";
import logo from "src/assets/images/logo.svg";

function WorkspaceMenuItem({
  membership,
  isCurrent,
}: {
  membership: TenantMembershipDto;
  isCurrent: boolean;
}) {
  const navigate = useNavigate();
  const handleSelect = useCallback(
    () => navigate(tenantPath(membership.tenant_id, APP_ROUTES.INDEX)),
    [navigate, membership.tenant_id],
  );
  return (
    <DropdownMenu.ItemWithDescription
      label={membership.tenant_name}
      description={membership.role_name}
      highlighted={isCurrent}
      onClick={handleSelect}
    />
  );
}

export function TenantSwitcher() {
  const navigate = useNavigate();
  const tenantId = useTenantId();
  const memberships = useMemberships();
  const currentMembership = useCurrentMembership();
  const handleCreateWorkspace = useCallback(
    () => navigate(APP_ROUTES.ONBOARDING),
    [navigate],
  );

  return (
    <DropdownMenu>
      <DropdownMenu.Trigger>
        <button
          type="button"
          className={SidebarStyles.link}
          aria-label={`Workspace: ${currentMembership.tenant_name}`}
          title={currentMembership.tenant_name}
        >
          <img src={logo} alt="" className={SidebarStyles.logo} />
        </button>
      </DropdownMenu.Trigger>
      <DropdownMenu.Content side="right" align="start">
        {memberships.data?.map((membership) => (
          <WorkspaceMenuItem
            key={membership.tenant_id}
            membership={membership}
            isCurrent={membership.tenant_id === tenantId}
          />
        ))}
        <DropdownMenu.Item icon="add" onClick={handleCreateWorkspace}>
          Create workspace
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu>
  );
}
