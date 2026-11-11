import {
  keepPreviousData,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { EmailStatus } from "src/api/invitation/queries";
import { AccountStatus } from "src/api/tenant/queries";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { ApiService } from "src/utils/api_client";

export type PlatformUserTenantDto = {
  tenant_id: string;
  tenant_name: string;
  role_name: string;
  role_display_name: string;
};

export type PlatformUserCapabilities = {
  can_deactivate: boolean;
  can_reactivate: boolean;
  can_send_password_reset: boolean;
  can_appoint_owner: boolean;
  can_make_administrator: boolean;
  can_remove_administrator: boolean;
};

export type PlatformUserDto = {
  id: string;
  full_name: string;
  email: string;
  is_current_user: boolean;
  is_platform_administrator: boolean;
  status: AccountStatus;
  has_password: boolean;
  provider_names: string[];
  tenants: PlatformUserTenantDto[];
  created_at: string;
  capabilities: PlatformUserCapabilities;
};

export type PlatformUserPageDto = {
  users: PlatformUserDto[];
  page: number;
  page_size: number;
  total: number;
  total_pages: number;
};

export type TenantSummaryDto = {
  tenant_id: string;
  tenant_name: string;
};

export type DeactivatedUserDto = {
  tenants_without_active_owner: TenantSummaryDto[];
};

export type PlatformTenantDto = {
  tenant_id: string;
  name: string;
  active_owner_count: number;
  created_at: string;
};

export type EmailLogEntryDto = {
  id: string;
  kind: string;
  kind_display_name: string;
  recipient: string;
  subject: string;
  status: EmailStatus;
  attempt_count: number;
  last_error: string | null;
  created_at: string;
  next_attempt_at: string;
  sent_at: string | null;
  can_retry: boolean;
};

export type EmailLogPageDto = {
  emails: EmailLogEntryDto[];
  page: number;
  page_size: number;
  total: number;
  total_pages: number;
};

export type PlatformUserQuery = {
  search: string;
  page: number;
};

export type EmailLogQuery = {
  status: EmailStatus | null;
  search: string;
  page: number;
};

export type AppointOwnerDto = {
  tenantId: string;
  userId: string;
};

export const platformKeys = {
  all: ["platform"] as const,
  users: (query: PlatformUserQuery) =>
    [...platformKeys.all, "users", query] as const,
  tenants: () => [...platformKeys.all, "tenants"] as const,
  emails: (query: EmailLogQuery) =>
    [...platformKeys.all, "emails", query] as const,
};

function queryString(parameters: Record<string, string | number | null>) {
  const searchParameters = new URLSearchParams();
  for (const [name, value] of Object.entries(parameters)) {
    if (value !== null && value !== "") {
      searchParameters.set(name, String(value));
    }
  }
  return searchParameters.toString();
}

const platformApi = {
  listUsers: async (query: PlatformUserQuery): Promise<PlatformUserPageDto> => {
    const response = await ApiService.get({
      url: `${API_ENDPOINTS.PLATFORM.USERS}?${queryString({ search: query.search.trim(), page: query.page })}`,
      onlyBody: true,
    });
    return response as PlatformUserPageDto;
  },

  deactivateUser: async (userId: string): Promise<DeactivatedUserDto> => {
    const response = await ApiService.post({
      url: API_ENDPOINTS.PLATFORM.DEACTIVATE_USER.replace(":user_id", userId),
    });
    return response as DeactivatedUserDto;
  },

  makeAdministrator: async (userId: string) => {
    await ApiService.post({
      url: API_ENDPOINTS.PLATFORM.MAKE_ADMINISTRATOR.replace(
        ":user_id",
        userId,
      ),
      onlyBody: false,
    });
  },

  removeAdministrator: async (userId: string) => {
    await ApiService.post({
      url: API_ENDPOINTS.PLATFORM.REMOVE_ADMINISTRATOR.replace(
        ":user_id",
        userId,
      ),
      onlyBody: false,
    });
  },

  reactivateUser: async (userId: string) => {
    await ApiService.post({
      url: API_ENDPOINTS.PLATFORM.REACTIVATE_USER.replace(":user_id", userId),
      onlyBody: false,
    });
  },

  sendPasswordReset: async (userId: string) => {
    await ApiService.post({
      url: API_ENDPOINTS.PLATFORM.SEND_PASSWORD_RESET.replace(
        ":user_id",
        userId,
      ),
      onlyBody: false,
    });
  },

  listTenants: async (): Promise<PlatformTenantDto[]> => {
    const response = await ApiService.get({
      url: API_ENDPOINTS.PLATFORM.TENANTS,
      onlyBody: true,
    });
    return response as PlatformTenantDto[];
  },

  appointOwner: async ({ tenantId, userId }: AppointOwnerDto) => {
    await ApiService.put({
      url: API_ENDPOINTS.PLATFORM.APPOINT_OWNER.replace(
        ":tenant_id",
        tenantId,
      ).replace(":user_id", userId),
      onlyBody: false,
    });
  },

  listEmails: async (query: EmailLogQuery): Promise<EmailLogPageDto> => {
    const response = await ApiService.get({
      url: `${API_ENDPOINTS.PLATFORM.EMAILS}?${queryString({ status: query.status, search: query.search.trim(), page: query.page })}`,
      onlyBody: true,
    });
    return response as EmailLogPageDto;
  },

  retryEmail: async (emailId: string) => {
    await ApiService.post({
      url: API_ENDPOINTS.PLATFORM.RETRY_EMAIL.replace(":email_id", emailId),
      onlyBody: false,
    });
  },
};

function useInvalidatePlatform() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: platformKeys.all });
}

export const usePlatformUsers = (query: PlatformUserQuery) => {
  return useQuery({
    queryKey: platformKeys.users(query),
    queryFn: () => platformApi.listUsers(query),
    placeholderData: keepPreviousData,
  });
};

export const useDeactivateUser = () => {
  const invalidatePlatform = useInvalidatePlatform();
  return useMutation({
    mutationFn: platformApi.deactivateUser,
    onSuccess: invalidatePlatform,
  });
};

export const useReactivateUser = () => {
  const invalidatePlatform = useInvalidatePlatform();
  return useMutation({
    mutationFn: platformApi.reactivateUser,
    onSuccess: invalidatePlatform,
  });
};

export const useMakeAdministrator = () => {
  const invalidatePlatform = useInvalidatePlatform();
  return useMutation({
    mutationFn: platformApi.makeAdministrator,
    onSuccess: invalidatePlatform,
  });
};

export const useRemoveAdministrator = () => {
  const invalidatePlatform = useInvalidatePlatform();
  return useMutation({
    mutationFn: platformApi.removeAdministrator,
    onSuccess: invalidatePlatform,
  });
};

export const useSendPasswordReset = () => {
  return useMutation({ mutationFn: platformApi.sendPasswordReset });
};

export const usePlatformTenants = () => {
  return useQuery({
    queryKey: platformKeys.tenants(),
    queryFn: platformApi.listTenants,
  });
};

export const useAppointOwner = () => {
  const invalidatePlatform = useInvalidatePlatform();
  return useMutation({
    mutationFn: platformApi.appointOwner,
    onSuccess: invalidatePlatform,
  });
};

export const useEmailLog = (query: EmailLogQuery) => {
  return useQuery({
    queryKey: platformKeys.emails(query),
    queryFn: () => platformApi.listEmails(query),
    placeholderData: keepPreviousData,
  });
};

export const useRetryEmail = () => {
  const invalidatePlatform = useInvalidatePlatform();
  return useMutation({
    mutationFn: platformApi.retryEmail,
    onSuccess: invalidatePlatform,
  });
};
