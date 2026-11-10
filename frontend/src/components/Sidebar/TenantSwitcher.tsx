import { useCallback } from "react";
import { useNavigate } from "react-router";
import {
  CollapsibleSidebar,
  DropdownMenu,
  useCollapsibleSidebar,
} from "src/components/design-system";
import { useCurrentMembership, useMemberships } from "src/api/tenant/queries";
import { APP_ROUTES } from "src/constants/app_routes";
import { useReceivedInvitations } from "src/api/invitation/queries";
import { tenantPath, useTenantId } from "src/utils/tenant";
import logo from "src/assets/images/logo.svg";

const LOGO_IMAGE = { src: logo, alt: "" };

export function TenantSwitcher() {
  const navigate = useNavigate();
  const tenantId = useTenantId();
  const memberships = useMemberships();
  const currentMembership = useCurrentMembership();
  const receivedInvitations = useReceivedInvitations();
  const { setIsHeldOpen } = useCollapsibleSidebar();

  const handleSelectTenant = useCallback(
    (selectedTenantId: string) => {
      void navigate(tenantPath(selectedTenantId, APP_ROUTES.INDEX));
    },
    [navigate],
  );
  const handleOpenOnboarding = useCallback(() => {
    void navigate(APP_ROUTES.ONBOARDING);
  }, [navigate]);

  const invitationCount = receivedInvitations.data?.length ?? 0;

  return (
    <DropdownMenu onOpenChange={setIsHeldOpen}>
      <DropdownMenu.Trigger>
        <CollapsibleSidebar.HeaderButton
          image={LOGO_IMAGE}
          label={currentMembership.tenant_name}
          trailingIcon="chevrons_up_down"
          aria-label={`Tenant: ${currentMembership.tenant_name}`}
        />
      </DropdownMenu.Trigger>
      <DropdownMenu.Content side="bottom" align="start" minWidthOfTrigger>
        <DropdownMenu.RadioGroup
          value={tenantId}
          onValueChange={handleSelectTenant}
        >
          {memberships.data?.map((membership) => (
            <DropdownMenu.RadioItem
              key={membership.tenant_id}
              value={membership.tenant_id}
            >
              {membership.tenant_name}
            </DropdownMenu.RadioItem>
          ))}
        </DropdownMenu.RadioGroup>
        {invitationCount > 0 && (
          <>
            <DropdownMenu.Separator />
            <DropdownMenu.Item icon="email" onClick={handleOpenOnboarding}>
              {`Invitations (${invitationCount})`}
            </DropdownMenu.Item>
          </>
        )}
      </DropdownMenu.Content>
    </DropdownMenu>
  );
}
