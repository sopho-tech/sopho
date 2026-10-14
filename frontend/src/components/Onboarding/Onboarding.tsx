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
} from "src/api/auth_api";
import { APP_ROUTES } from "src/constants/app_routes";
import { PLATFORM_PERMISSIONS } from "src/constants/platform_permissions";
import { CreateWorkspaceForm } from "src/components/Onboarding/CreateWorkspaceForm";
import { ReceivedInvitations } from "src/components/Onboarding/ReceivedInvitations";

export function Onboarding() {
  const navigate = useNavigate();
  const signedInUser = useSignedInUser();
  const memberships = useMemberships();
  const receivedInvitations = useReceivedInvitations();
  const canCreateWorkspace = usePlatformPermission(
    PLATFORM_PERMISSIONS.TENANT_CREATE,
  );
  const signOut = useDeleteSession();
  const { mutate: deleteSession } = signOut;

  const handleBack = useCallback(() => navigate(APP_ROUTES.INDEX), [navigate]);
  const handleSignOut = useCallback(() => deleteSession(), [deleteSession]);

  const hasWorkspaces = (memberships.data?.length ?? 0) > 0;
  const invitations = receivedInvitations.data;
  const hasNothingToJoin =
    invitations !== undefined &&
    invitations.length === 0 &&
    !canCreateWorkspace;

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
          <Heading accessbilityLevel={1} size="xl" textAlign="center">
            {hasWorkspaces ? "Join or create a workspace" : "Welcome to Sopho"}
          </Heading>
          <Heading
            accessbilityLevel={2}
            size="sm"
            textAlign="center"
            textColor="subtle"
          >
            Canvases, conversations and connections live inside a workspace
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
        {canCreateWorkspace && <CreateWorkspaceForm />}
        {hasNothingToJoin && (
          <Text fontSize="sm" color="subtle">
            {`You are not part of a workspace yet. Ask a workspace admin to invite ${signedInUser.email}.`}
          </Text>
        )}
        <Flex direction="row" justifyContent="space-between" gap="sm">
          {hasWorkspaces ? (
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
