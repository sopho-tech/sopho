import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { useTenantId } from "src/utils/tenant";
import { Permission } from "src/constants/permissions";
import { platformKeys } from "src/api/platform/queries";

export type TenantMembershipDto = {
  tenant_id: string;
  tenant_name: string;
  role_name: string;
  role_display_name: string;
  permissions: string[];
};

export type AccountStatus = "ACTIVE" | "DEACTIVATED";

export type TenantMemberDto = {
  user_id: string;
  full_name: string;
  email: string;
  role_name: string;
  role_display_name: string;
  account_status: AccountStatus;
};

export type CreatedTenantDto = {
  id: string;
  name: string;
};

export type TenantCreationCapabilityDto = {
  can_create_tenant: boolean;
  limit_reached_message: string | null;
};

export const tenantKeys = {
  memberships: ["memberships"] as const,
  creationCapability: ["tenant-creation-capability"] as const,
};

const tenantApi = {
  listMemberships: async (): Promise<TenantMembershipDto[]> => {
    const response = await ApiService.get({
      url: API_ENDPOINTS.TENANT.LIST,
      onlyBody: true,
    });
    return response as TenantMembershipDto[];
  },

  getCreationCapability: async (): Promise<TenantCreationCapabilityDto> => {
    const response = await ApiService.get({
      url: API_ENDPOINTS.TENANT.CREATION_CAPABILITY,
      onlyBody: true,
    });
    return response as TenantCreationCapabilityDto;
  },

  createTenant: async (name: string): Promise<CreatedTenantDto> => {
    const response = await ApiService.post({
      url: API_ENDPOINTS.TENANT.CREATE,
      data: { name },
      headers: { "Content-Type": "application/json" },
    });
    return response as CreatedTenantDto;
  },
};

export const useMemberships = () => {
  return useQuery({
    queryKey: tenantKeys.memberships,
    queryFn: tenantApi.listMemberships,
  });
};

export const useTenantCreationCapability = () => {
  return useQuery({
    queryKey: tenantKeys.creationCapability,
    queryFn: tenantApi.getCreationCapability,
  });
};

export const useCurrentMembership = (): TenantMembershipDto => {
  const tenantId = useTenantId();
  const memberships = useMemberships();
  const membership = memberships.data?.find(
    (candidate) => candidate.tenant_id === tenantId,
  );
  if (!membership) {
    throw new Error("The current tenant membership is not loaded");
  }
  return membership;
};

export const useTenantPermission = (permission: Permission): boolean =>
  useCurrentMembership().permissions.includes(permission);

export const useCreateTenant = (callbacks?: {
  onSuccess?: (tenant: CreatedTenantDto) => void;
}) => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: tenantApi.createTenant,
    onSuccess: async (tenant) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: tenantKeys.memberships }),
        queryClient.invalidateQueries({
          queryKey: tenantKeys.creationCapability,
        }),
        queryClient.invalidateQueries({ queryKey: platformKeys.all }),
      ]);
      callbacks?.onSuccess?.(tenant);
    },
  });
};
