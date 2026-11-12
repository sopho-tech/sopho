import { useCallback, useState } from "react";
import { useNavigate } from "react-router";
import {
  BannerSlim,
  Button,
  Flex,
  Heading,
  Text,
} from "src/components/design-system";
import {
  CreatedTenantDto,
  useCreateTenant,
  useMemberships,
  useTenantCreationCapability,
} from "src/api/tenant/queries";
import { useReceivedInvitations } from "src/api/invitation/queries";
import { useDeleteSession, useSignedInUser } from "src/api/auth/queries";
import { AuthPageLayout } from "src/components/AuthPageLayout";
import { CreateTenantDialog } from "src/components/CreateTenantDialog";
import { CreateFirstTenantForm } from "src/components/Onboarding/CreateFirstTenantForm";
import { ReceivedInvitations } from "src/components/Onboarding/ReceivedInvitations";
import { TenantList } from "src/components/Onboarding/TenantList";
import { APP_ROUTES } from "src/constants/app_routes";
import { tenantPath } from "src/utils/tenant";

const TENANT_LISTS_STYLE = { maxHeight: "50vh" };

export function Onboarding() {
  const signedInUser = useSignedInUser();
  const memberships = useMemberships();
  const receivedInvitations = useReceivedInvitations();
  const tenantCreationCapability = useTenantCreationCapability();
  const signOut = useDeleteSession();
  const { mutate: deleteSession } = signOut;
  const navigate = useNavigate();
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);

  const handleSignOut = useCallback(() => deleteSession(), [deleteSession]);
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
      void navigate(tenantPath(tenant.id, APP_ROUTES.INDEX));
    },
    [navigate],
  );

  const createFirstTenant = useCreateTenant({ onSuccess: handleTenantCreated });
  const { mutate: submitFirstTenant } = createFirstTenant;
  const handleCreateFirstTenant = useCallback(
    (name: string) => submitFirstTenant(name),
    [submitFirstTenant],
  );
  const isOpeningFirstTenant =
    createFirstTenant.isPending || createFirstTenant.isSuccess;

  const hasTenants =
    memberships.data !== undefined && memberships.data.length > 0;
  const hasInvitations =
    receivedInvitations.data !== undefined &&
    receivedInvitations.data.length > 0;
  const hasNothingToJoin =
    memberships.data !== undefined &&
    receivedInvitations.data !== undefined &&
    !hasTenants &&
    !hasInvitations;
  const canCreateTenant =
    tenantCreationCapability.data?.can_create_tenant === true;
  const limitReachedMessage =
    tenantCreationCapability.data?.limit_reached_message;
  const mustAskForInvitation =
    hasNothingToJoin &&
    tenantCreationCapability.data !== undefined &&
    !canCreateTenant &&
    !limitReachedMessage;

  const showsFirstTenantForm =
    (hasNothingToJoin && canCreateTenant) || isOpeningFirstTenant;
  const signOutButton = (
    <Button
      label="Sign out"
      shape="rectangle"
      size="sm"
      backgroundColor="transparent"
      onClick={handleSignOut}
      disabled={signOut.isPending}
    />
  );

  return (
    <AuthPageLayout>
      <Flex direction="column" alignItems="center" gap="2xs">
        <Heading accessibilityLevel={1} size="xl" textAlign="center">
          {hasTenants && !isOpeningFirstTenant
            ? "Choose a tenant"
            : "Welcome to Sopho"}
        </Heading>
        <Text fontSize="sm" color="subtle">
          {`Signed in as ${signedInUser.email}`}
        </Text>
      </Flex>
      <Flex direction="column" gap="lg" paddingTop="lg">
        {memberships.error && (
          <BannerSlim type="error" message={memberships.error.message} />
        )}
        {receivedInvitations.error && (
          <BannerSlim
            type="error"
            message={receivedInvitations.error.message}
          />
        )}
        {tenantCreationCapability.error && (
          <BannerSlim
            type="error"
            message={tenantCreationCapability.error.message}
          />
        )}
        {!isOpeningFirstTenant && (hasInvitations || hasTenants) && (
          <Flex
            direction="column"
            gap="lg"
            overflow="scrollY"
            sx={TENANT_LISTS_STYLE}
          >
            {hasInvitations && (
              <ReceivedInvitations invitations={receivedInvitations.data} />
            )}
            {hasTenants && <TenantList memberships={memberships.data} />}
          </Flex>
        )}
        {showsFirstTenantForm && (
          <CreateFirstTenantForm
            onCreate={handleCreateFirstTenant}
            isCreating={isOpeningFirstTenant}
            errorMessage={createFirstTenant.error?.message ?? null}
            leadingAction={signOutButton}
          />
        )}
        {!isOpeningFirstTenant && limitReachedMessage && (
          <BannerSlim type="info" message={limitReachedMessage} />
        )}
        {mustAskForInvitation && (
          <BannerSlim
            type="info"
            message={`You are not part of a tenant yet. Ask a tenant admin to invite ${signedInUser.email}.`}
          />
        )}
        {!showsFirstTenantForm && (
          <Flex
            direction="row"
            justifyContent="space-between"
            alignItems="center"
            gap="sm"
          >
            {signOutButton}
            {!hasNothingToJoin && canCreateTenant && (
              <Button
                label="Create a tenant"
                leadingIconName="add"
                shape="rectangle"
                size="sm"
                backgroundColor="white"
                onClick={handleOpenCreateDialog}
              />
            )}
          </Flex>
        )}
      </Flex>
      {isCreateDialogOpen && (
        <CreateTenantDialog
          onCreated={handleTenantCreated}
          onClose={handleCloseCreateDialog}
        />
      )}
    </AuthPageLayout>
  );
}
