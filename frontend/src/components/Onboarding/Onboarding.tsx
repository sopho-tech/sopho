import { useCallback } from "react";
import {
  BannerSlim,
  Button,
  Flex,
  Heading,
  Text,
} from "src/components/design-system";
import { useMemberships } from "src/api/tenant/queries";
import { useReceivedInvitations } from "src/api/invitation/queries";
import { useDeleteSession, useSignedInUser } from "src/api/auth/queries";
import { AuthPageLayout } from "src/components/AuthPageLayout";
import { ReceivedInvitations } from "src/components/Onboarding/ReceivedInvitations";
import { TenantList } from "src/components/Onboarding/TenantList";

const TENANT_LISTS_STYLE = { maxHeight: "50vh" };

export function Onboarding() {
  const signedInUser = useSignedInUser();
  const memberships = useMemberships();
  const receivedInvitations = useReceivedInvitations();
  const signOut = useDeleteSession();
  const { mutate: deleteSession } = signOut;

  const handleSignOut = useCallback(() => deleteSession(), [deleteSession]);

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

  return (
    <AuthPageLayout>
      <Flex direction="column" alignItems="center" gap="2xs">
        <Heading accessibilityLevel={1} size="xl" textAlign="center">
          {hasTenants ? "Choose a tenant" : "Welcome to Sopho"}
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
        {(hasInvitations || hasTenants) && (
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
        {hasNothingToJoin && (
          <BannerSlim
            type="info"
            message={`You are not part of a tenant yet. Ask a tenant admin to invite ${signedInUser.email}.`}
          />
        )}
        <Flex direction="row" justifyContent="flex-end">
          <Button
            label="Sign out"
            shape="rectangle"
            size="sm"
            backgroundColor="transparent"
            onClick={handleSignOut}
            disabled={signOut.isPending}
          />
        </Flex>
      </Flex>
    </AuthPageLayout>
  );
}
