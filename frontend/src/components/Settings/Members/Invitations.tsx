import { useState } from "react";
import { Navigate, useLocation } from "react-router";
import { BannerSlim, Flex, Heading, Text } from "src/components/design-system";
import { useTenantPermission } from "src/api/tenant/queries";
import {
  useInvitationDelivery,
  useTenantInvitations,
} from "src/api/invitation/queries";
import { APP_ROUTES } from "src/constants/app_routes";
import { PERMISSIONS } from "src/constants/permissions";
import { InvitationsTable } from "src/components/Settings/Members/InvitationsTable";
import { InvitePeopleButton } from "src/components/Settings/Members/InvitePeopleButton";
import { useTenantPath } from "src/utils/tenant";

export type InvitationsLocationState = {
  notice: string;
};

export function Invitations() {
  const toTenantPath = useTenantPath();
  const location = useLocation();
  const canInvite = useTenantPermission(PERMISSIONS.MEMBER_INVITE);
  const invitationDelivery = useInvitationDelivery(canInvite);
  const invitations = useTenantInvitations(canInvite);
  const locationState = location.state as InvitationsLocationState | null;
  const [notice, setNotice] = useState<string | null>(
    locationState ? locationState.notice : null,
  );

  if (!canInvite) {
    return (
      <Navigate to={toTenantPath(APP_ROUTES.SETTINGS_ROUTES.MEMBERS)} replace />
    );
  }

  const failure = invitationDelivery.error ?? invitations.error;

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
            Invitations
          </Heading>
          <Text fontSize="sm" color="subtle">
            Invite people to this tenant and follow up on the invitations that
            are still open.
          </Text>
        </Flex>
        <InvitePeopleButton onInvited={setNotice} />
      </Flex>
      {failure && <BannerSlim type="error" message={failure.message} />}
      {invitationDelivery.data?.unavailable_reason && (
        <BannerSlim
          type="info"
          message={invitationDelivery.data.unavailable_reason}
        />
      )}
      {notice && <BannerSlim type="success" message={notice} />}
      {invitations.data && (
        <InvitationsTable
          invitations={invitations.data}
          canResend={invitationDelivery.data?.method === "EMAIL"}
        />
      )}
    </Flex>
  );
}
