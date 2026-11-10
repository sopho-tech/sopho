import { useCallback, useMemo } from "react";
import { Button } from "src/components/design-system";
import { OidcProviderDto } from "src/api/auth/queries";
import { PROVIDER_LOGOS } from "src/components/SignIn/constants";
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
  const logo = PROVIDER_LOGOS[provider.icon];
  const leadingImage = useMemo(
    () => (logo ? { src: logo, alt: "" } : undefined),
    [logo],
  );

  return (
    <Button
      label={provider.display_name}
      shape="rectangle"
      size="md"
      backgroundColor="white"
      fullWidth
      outlined
      contentAlignment="center"
      leadingImage={leadingImage}
      leadingIconName={logo ? undefined : "key"}
      onClick={handleClick}
    />
  );
}
