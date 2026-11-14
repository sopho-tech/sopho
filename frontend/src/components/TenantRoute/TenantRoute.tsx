import { Fragment } from "react";
import {
  invitationPath,
  pendingInvitationToken,
} from "src/components/InvitationLink/pending_invitation";
import { Navigate, Outlet } from "react-router";
import { isUnauthorizedError } from "src/api/dto";
import { useReceivedInvitations } from "src/api/invitation/queries";
import { useMemberships } from "src/api/tenant/queries";
import { Sidebar } from "src/components/Sidebar/Sidebar";
import { SuspendedTenant } from "src/components/SuspendedTenant";
import { CommandMenu } from "src/components/design-system/CommandMenu/CommandMenu";
import { Flex } from "src/components/design-system";
import { APP_ROUTES } from "src/constants/app_routes";
import { tenantPath, useTenantId } from "src/utils/tenant";

export function TenantRoute() {
  const tenantId = useTenantId();
  const memberships = useMemberships();

  if (memberships.isPending) return null;
  if (memberships.isError) {
    if (isUnauthorizedError(memberships.error)) return null;
    throw memberships.error;
  }

  const currentMembership = memberships.data.find(
    (membership) => membership.tenant_id === tenantId,
  );
  if (!currentMembership) {
    return <Navigate to={APP_ROUTES.INDEX} replace />;
  }
  if (currentMembership.suspended_message !== null) {
    return (
      <SuspendedTenant
        tenantName={currentMembership.tenant_name}
        suspendedMessage={currentMembership.suspended_message}
      />
    );
  }

  return (
    <Fragment>
      <Flex direction="row" height="100vh" width="100vw">
        <Sidebar />
        <Flex flex="40" overflow="hidden">
          <Outlet />
        </Flex>
      </Flex>
      <CommandMenu />
    </Fragment>
  );
}

export function TenantIndexRedirect() {
  const memberships = useMemberships();
  const receivedInvitations = useReceivedInvitations();
  const pendingToken = pendingInvitationToken();

  if (pendingToken) {
    return <Navigate to={invitationPath(pendingToken)} replace />;
  }
  if (memberships.isPending || receivedInvitations.isPending) return null;
  if (memberships.isError) {
    if (isUnauthorizedError(memberships.error)) return null;
    throw memberships.error;
  }
  if (receivedInvitations.isError) {
    if (isUnauthorizedError(receivedInvitations.error)) return null;
    throw receivedInvitations.error;
  }

  const [onlyMembership] = memberships.data;
  const hasNothingToChoose =
    memberships.data.length === 1 && receivedInvitations.data.length === 0;
  if (!hasNothingToChoose) {
    return <Navigate to={APP_ROUTES.ONBOARDING} replace />;
  }
  return (
    <Navigate
      to={tenantPath(onlyMembership.tenant_id, APP_ROUTES.INDEX)}
      replace
    />
  );
}
