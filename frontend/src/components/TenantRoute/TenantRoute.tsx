import { Fragment } from "react";
import {
  invitationPath,
  pendingInvitationToken,
} from "src/utils/pending_invitation";
import { Navigate, Outlet } from "react-router";
import { isUnauthorizedError } from "src/api/dto";
import { useMemberships } from "src/api/tenant/queries";
import { Sidebar } from "src/components/Sidebar/Sidebar";
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

  const isMember = memberships.data.some(
    (membership) => membership.tenant_id === tenantId,
  );
  if (!isMember) {
    return <Navigate to={APP_ROUTES.INDEX} replace />;
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
  const pendingToken = pendingInvitationToken();

  if (pendingToken) {
    return <Navigate to={invitationPath(pendingToken)} replace />;
  }
  if (memberships.isPending) return null;
  if (memberships.isError) {
    if (isUnauthorizedError(memberships.error)) return null;
    throw memberships.error;
  }

  const [firstMembership] = memberships.data;
  if (!firstMembership) {
    return <Navigate to={APP_ROUTES.ONBOARDING} replace />;
  }
  return (
    <Navigate
      to={tenantPath(firstMembership.tenant_id, APP_ROUTES.INDEX)}
      replace
    />
  );
}
