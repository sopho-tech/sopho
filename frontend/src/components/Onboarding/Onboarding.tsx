import { useCallback } from "react";
import { useNavigate } from "react-router";
import {
  BannerSlim,
  Button,
  Flex,
  Heading,
  Text,
} from "src/components/design-system";
import { useMemberships } from "src/api/tenant/queries";
import { useReceivedInvitations } from "src/api/invitation/queries";
import {
  useDeleteSession,
  usePlatformPermission,
  useSignedInUser,
} from "src/api/auth/queries";
import { APP_ROUTES } from "src/constants/app_routes";
import { PLATFORM_PERMISSIONS } from "src/constants/platform_permissions";
import { CreateTenantForm } from "src/components/Onboarding/CreateTenantForm";
import { ReceivedInvitations } from "src/components/Onboarding/ReceivedInvitations";

export function Onboarding() {
  const navigate = useNavigate();
  const signedInUser = useSignedInUser();
  const memberships = useMemberships();
  const receivedInvitations = useReceivedInvitations();
  const canCreateTenant = usePlatformPermission(
    PLATFORM_PERMISSIONS.TENANT_CREATE,
  );
  const signOut = useDeleteSession();
  const { mutate: deleteSession } = signOut;

  const handleBack = useCallback(() => {
    void navigate(APP_ROUTES.INDEX);
  }, [navigate]);
  const handleSignOut = useCallback(() => deleteSession(), [deleteSession]);

  const hasTenants = (memberships.data?.length ?? 0) > 0;
  const invitations = receivedInvitations.data;
  const hasNothingToJoin =
    invitations !== undefined && invitations.length === 0 && !canCreateTenant;

  return (
    <Flex
      width="100vw"
      height="100vh"
      justifyContent="center"
      alignItems="center"
    >
      <Flex
        direction="column"
        shadow="sm"
        borderRadius="lg"
        paddingX="xl"
        paddingY="xl"
        gap="lg"
        backgroundColor="white"
      >
        <Flex direction="column" gap="xs">
          <Heading accessibilityLevel={1} size="xl" textAlign="center">
            {hasTenants ? "Join or create a tenant" : "Welcome to Sopho"}
          </Heading>
          <Heading
            accessibilityLevel={2}
            size="sm"
            textAlign="center"
            textColor="subtle"
          >
            Canvases, conversations and connections live inside a tenant
          </Heading>
        </Flex>
        {memberships.error && (
          <BannerSlim type="error" message={memberships.error.message} />
        )}
        {receivedInvitations.error && (
          <BannerSlim
            type="error"
            message={receivedInvitations.error.message}
          />
        )}
        {invitations && invitations.length > 0 && (
          <ReceivedInvitations invitations={invitations} />
        )}
        {canCreateTenant && <CreateTenantForm />}
        {hasNothingToJoin && (
          <Text fontSize="sm" color="subtle">
            {`You are not part of a tenant yet. Ask a tenant admin to invite ${signedInUser.email}.`}
          </Text>
        )}
        <Flex direction="row" justifyContent="space-between" gap="sm">
          {hasTenants ? (
            <Button
              label="Back"
              shape="rectangle"
              size="md"
              backgroundColor="transparent"
              onClick={handleBack}
            />
          ) : (
            <Flex />
          )}
          <Button
            label="Sign out"
            shape="rectangle"
            size="md"
            backgroundColor="transparent"
            onClick={handleSignOut}
            disabled={signOut.isPending}
          />
        </Flex>
      </Flex>
    </Flex>
  );
}
