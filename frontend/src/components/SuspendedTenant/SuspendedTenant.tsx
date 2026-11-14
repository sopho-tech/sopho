import { useCallback } from "react";
import { useNavigate } from "react-router";
import { AuthPageLayout } from "src/components/AuthPageLayout";
import { BACK_TO_TENANTS_LABEL } from "src/components/SuspendedTenant/constants";
import {
  BannerSlim,
  Button,
  Flex,
  Heading,
} from "src/components/design-system";
import { APP_ROUTES } from "src/constants/app_routes";

type SuspendedTenantProps = {
  tenantName: string;
  suspendedMessage: string;
};

export function SuspendedTenant({
  tenantName,
  suspendedMessage,
}: SuspendedTenantProps) {
  const navigate = useNavigate();
  const handleBackToTenants = useCallback(() => {
    void navigate(APP_ROUTES.ONBOARDING);
  }, [navigate]);

  return (
    <AuthPageLayout>
      <Flex direction="column" gap="lg">
        <Heading accessibilityLevel={1} size="xl" textColor="black">
          {tenantName}
        </Heading>
        <BannerSlim type="warning" message={suspendedMessage} />
        <Flex direction="row" justifyContent="flex-start">
          <Button
            label={BACK_TO_TENANTS_LABEL}
            leadingIconName="chevron_left"
            shape="rectangle"
            size="sm"
            backgroundColor="white"
            onClick={handleBackToTenants}
          />
        </Flex>
      </Flex>
    </AuthPageLayout>
  );
}
