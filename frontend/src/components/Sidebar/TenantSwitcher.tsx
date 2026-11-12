import { useCallback, useState } from "react";
import { useNavigate } from "react-router";
import {
  CollapsibleSidebar,
  DropdownMenu,
  useCollapsibleSidebar,
} from "src/components/design-system";
import {
  CreatedTenantDto,
  useCurrentMembership,
  useMemberships,
  useTenantCreationCapability,
} from "src/api/tenant/queries";
import { CreateTenantDialog } from "src/components/CreateTenantDialog";
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
  const tenantCreationCapability = useTenantCreationCapability();
  const { setIsHeldOpen } = useCollapsibleSidebar();
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);

  const handleSelectTenant = useCallback(
    (selectedTenantId: string) => {
      void navigate(tenantPath(selectedTenantId, APP_ROUTES.INDEX));
    },
    [navigate],
  );
  const handleOpenOnboarding = useCallback(() => {
    void navigate(APP_ROUTES.ONBOARDING);
  }, [navigate]);

  const handleOpenCreateDialog = useCallback(
    () => setIsCreateDialogOpen(true),
    [],
  );
  const handleCloseCreateDialog = useCallback(
    () => setIsCreateDialogOpen(false),
    [],
  );
  const handleTenantCreated = useCallback(
    (tenant: CreatedTenantDto) => {
      setIsCreateDialogOpen(false);
      void navigate(tenantPath(tenant.id, APP_ROUTES.INDEX));
    },
    [navigate],
  );

  const invitationCount = receivedInvitations.data?.length ?? 0;
  const canCreateTenant =
    tenantCreationCapability.data?.can_create_tenant === true;

  return (
    <>
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
          {canCreateTenant && (
            <>
              <DropdownMenu.Separator />
              <DropdownMenu.Item icon="add" onClick={handleOpenCreateDialog}>
                Create a tenant
              </DropdownMenu.Item>
            </>
          )}
        </DropdownMenu.Content>
      </DropdownMenu>
      {isCreateDialogOpen && (
        <CreateTenantDialog
          onCreated={handleTenantCreated}
          onClose={handleCloseCreateDialog}
        />
      )}
    </>
  );
}
