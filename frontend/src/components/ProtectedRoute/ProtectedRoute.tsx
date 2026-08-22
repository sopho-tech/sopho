import { Fragment } from "react";
import { Navigate, Outlet } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import { useSessionValid } from "src/api/auth_api";
import { Sidebar } from "src/components/Sidebar/Sidebar";
import { CommandMenu } from "src/components/design-system/CommandMenu/CommandMenu";
import { Flex } from "../design-system/Flex/Flex";

export function ProtectedRoute() {
  const { data: isAuthenticated, isLoading } = useSessionValid();

  const component = (
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

  if (isLoading) return null;
  return isAuthenticated ? (
    component
  ) : (
    <Navigate to={APP_ROUTES.SIGN_IN} replace />
  );
}
