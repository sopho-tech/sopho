import { useCallback } from "react";
import { Button } from "src/components/design-system";
import { OidcProviderDto } from "src/api/auth/queries";
import { API_ENDPOINTS } from "src/constants/api_endpoints";

type SingleSignOnButtonProps = {
  provider: OidcProviderDto;
  onBeforeRedirect?: () => void;
};

export function SingleSignOnButton({
  provider,
  onBeforeRedirect,
}: SingleSignOnButtonProps) {
  const handleClick = useCallback(() => {
    onBeforeRedirect?.();
    window.location.assign(
      API_ENDPOINTS.AUTH.OIDC_START.replace(":provider_name", provider.name),
    );
  }, [onBeforeRedirect, provider.name]);
  return (
    <Button
      label={`Continue with ${provider.display_name}`}
      shape="rectangle"
      size="md"
      backgroundColor="white"
      fullWidth
      onClick={handleClick}
    />
  );
}
