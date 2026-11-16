import { EMAIL_TEMPLATE_KIND_PARAMETER } from "src/components/Platform/constants";
import { APP_ROUTES } from "src/constants/app_routes";

export function emailTemplatePath(emailKind: string): string {
  const searchParameters = new URLSearchParams({
    [EMAIL_TEMPLATE_KIND_PARAMETER]: emailKind,
  });
  return `${APP_ROUTES.PLATFORM_ROUTES.EMAIL_TEMPLATES}?${searchParameters.toString()}`;
}
