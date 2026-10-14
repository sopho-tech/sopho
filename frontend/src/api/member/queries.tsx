import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { tenantApiUrl, useTenantId } from "src/utils/tenant";
import { TenantMemberDto, tenantKeys } from "src/api/tenant/queries";

export type MemberCapabilities = {
  can_change_role: boolean;
  can_remove: boolean;
};

export type MemberDto = TenantMemberDto & {
  capabilities: MemberCapabilities;
};

export type RoleDto = {
  name: string;
  is_system: boolean;
  assignable: boolean;
};

export const memberKeys = {
  list: (tenantId: string) => [tenantId, "members"] as const,
  roles: (tenantId: string) => [tenantId, "roles"] as const,
};

const memberApi = {
  listMembers: async (tenantId: string): Promise<MemberDto[]> => {
    const response = await ApiService.get({
      url: tenantApiUrl(tenantId, API_ENDPOINTS.TENANT.MEMBERS),
      onlyBody: true,
    });
    return response as MemberDto[];
  },

  listRoles: async (tenantId: string): Promise<RoleDto[]> => {
    const response = await ApiService.get({
      url: tenantApiUrl(tenantId, API_ENDPOINTS.TENANT.ROLES),
      onlyBody: true,
    });
    return response as RoleDto[];
  },

  updateRole: async (
    tenantId: string,
    variables: { userId: string; roleName: string },
  ): Promise<MemberDto> => {
    const response = await ApiService.put({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.TENANT.MEMBER.replace(":user_id", variables.userId),
      ),
      data: { role_name: variables.roleName },
      headers: { "Content-Type": "application/json" },
    });
    return response as MemberDto;
  },

  remove: async (tenantId: string, userId: string): Promise<void> => {
    await ApiService.delete({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.TENANT.MEMBER.replace(":user_id", userId),
      ),
    });
  },
};

export const useTenantMembers = () => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: memberKeys.list(tenantId),
    queryFn: () => memberApi.listMembers(tenantId),
  });
};

export const useRoles = () => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: memberKeys.roles(tenantId),
    queryFn: () => memberApi.listRoles(tenantId),
  });
};

const useInvalidateMembership = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();
  return () =>
    Promise.all([
      queryClient.invalidateQueries({ queryKey: memberKeys.list(tenantId) }),
      queryClient.invalidateQueries({ queryKey: memberKeys.roles(tenantId) }),
      queryClient.invalidateQueries({ queryKey: tenantKeys.memberships }),
    ]);
};

export const useUpdateMemberRole = () => {
  const tenantId = useTenantId();
  const invalidateMembership = useInvalidateMembership();
  return useMutation({
    mutationFn: (variables: { userId: string; roleName: string }) =>
      memberApi.updateRole(tenantId, variables),
    onSuccess: invalidateMembership,
  });
};

export const useRemoveMember = () => {
  const tenantId = useTenantId();
  const invalidateMembership = useInvalidateMembership();
  return useMutation({
    mutationFn: (userId: string) => memberApi.remove(tenantId, userId),
    onSuccess: invalidateMembership,
  });
};
