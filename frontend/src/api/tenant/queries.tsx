import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { tenantApiUrl, useTenantId } from "src/utils/tenant";
import { Permission } from "src/constants/permissions";

export type TenantMembershipDto = {
  tenant_id: string;
  tenant_name: string;
  role_name: string;
  permissions: string[];
};

export type TenantMemberDto = {
  user_id: string;
  full_name: string;
  email: string;
  role_name: string;
};

export type CreatedTenantDto = {
  id: string;
  name: string;
};

export const tenantKeys = {
  memberships: ["memberships"] as const,
  members: (tenantId: string) => [tenantId, "members"] as const,
};

const tenantApi = {
  listMemberships: async (): Promise<TenantMembershipDto[]> => {
    const response = await ApiService.get({
      url: API_ENDPOINTS.TENANT.LIST,
      onlyBody: true,
    });
    return response as TenantMembershipDto[];
  },

  createTenant: async (name: string): Promise<CreatedTenantDto> => {
    const response = await ApiService.post({
      url: API_ENDPOINTS.TENANT.CREATE,
      data: { name },
      headers: { "Content-Type": "application/json" },
    });
    return response as CreatedTenantDto;
  },

  listMembers: async (tenantId: string): Promise<TenantMemberDto[]> => {
    const response = await ApiService.get({
      url: tenantApiUrl(tenantId, API_ENDPOINTS.TENANT.MEMBERS),
      onlyBody: true,
    });
    return response as TenantMemberDto[];
  },
};

export const useMemberships = () => {
  return useQuery({
    queryKey: tenantKeys.memberships,
    queryFn: tenantApi.listMemberships,
  });
};

export const useCurrentMembership = (): TenantMembershipDto => {
  const tenantId = useTenantId();
  const memberships = useMemberships();
  const membership = memberships.data?.find(
    (candidate) => candidate.tenant_id === tenantId,
  );
  if (!membership) {
    throw new Error("The current workspace membership is not loaded");
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
      await queryClient.invalidateQueries({
        queryKey: tenantKeys.memberships,
      });
      callbacks?.onSuccess?.(tenant);
    },
  });
};

export const useTenantMembers = () => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: tenantKeys.members(tenantId),
    queryFn: () => tenantApi.listMembers(tenantId),
  });
};
