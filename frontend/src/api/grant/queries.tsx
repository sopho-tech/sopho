import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { tenantApiUrl, useTenantId } from "src/utils/tenant";
import { TenantMemberDto } from "src/api/tenant/queries";

export type SharedResourceType = "CANVAS" | "CONVERSATION" | "CONNECTION";
export type ResourceVisibility = "PRIVATE" | "TENANT";
export type GrantAccessLevel = "VIEWER" | "EDITOR";

export type GrantDto = {
  id: string;
  resource_type: SharedResourceType;
  resource_id: string;
  principal_user_id: string;
  principal: TenantMemberDto | null;
  access_level: GrantAccessLevel;
};

export type ResourceSharingDto = {
  resource_type: SharedResourceType;
  resource_id: string;
  visibility: ResourceVisibility;
  owner: TenantMemberDto | null;
  grants: GrantDto[];
};

export const grantKeys = {
  sharing: (
    tenantId: string,
    resourceType: SharedResourceType,
    resourceId: string,
  ) => [tenantId, "sharing", resourceType, resourceId] as const,
};

const sharingPath = (resourceType: SharedResourceType, resourceId: string) =>
  API_ENDPOINTS.GRANT.SHARING.replace(":resource_type", resourceType).replace(
    ":resource_id",
    resourceId,
  );

const grantApi = {
  getSharing: async (
    tenantId: string,
    resourceType: SharedResourceType,
    resourceId: string,
  ): Promise<ResourceSharingDto> => {
    const response = await ApiService.get({
      url: tenantApiUrl(tenantId, sharingPath(resourceType, resourceId)),
      onlyBody: true,
    });
    return response as ResourceSharingDto;
  },

  updateVisibility: async (
    tenantId: string,
    resourceType: SharedResourceType,
    resourceId: string,
    visibility: ResourceVisibility,
  ): Promise<ResourceSharingDto> => {
    const response = await ApiService.put({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.GRANT.VISIBILITY.replace(
          ":resource_type",
          resourceType,
        ).replace(":resource_id", resourceId),
      ),
      data: { visibility },
      headers: { "Content-Type": "application/json" },
    });
    return response as ResourceSharingDto;
  },

  createGrant: async (
    tenantId: string,
    payload: {
      resource_type: SharedResourceType;
      resource_id: string;
      principal_user_id: string;
      access_level: GrantAccessLevel;
    },
  ): Promise<GrantDto> => {
    const response = await ApiService.post({
      url: tenantApiUrl(tenantId, API_ENDPOINTS.GRANT.CREATE),
      data: payload,
      headers: { "Content-Type": "application/json" },
    });
    return response as GrantDto;
  },

  revokeGrant: async (tenantId: string, grantId: string): Promise<void> => {
    await ApiService.delete({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.GRANT.REVOKE.replace(":grant_id", grantId),
      ),
    });
  },
};

export const useResourceSharing = (
  resourceType: SharedResourceType,
  resourceId: string,
  enabled: boolean,
) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: grantKeys.sharing(tenantId, resourceType, resourceId),
    queryFn: () => grantApi.getSharing(tenantId, resourceType, resourceId),
    enabled: enabled && !!resourceId,
  });
};

export const useUpdateVisibility = (
  resourceType: SharedResourceType,
  resourceId: string,
) => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();
  return useMutation({
    mutationFn: (visibility: ResourceVisibility) =>
      grantApi.updateVisibility(tenantId, resourceType, resourceId, visibility),
    onSuccess: (sharing) => {
      queryClient.setQueryData(
        grantKeys.sharing(tenantId, resourceType, resourceId),
        sharing,
      );
    },
  });
};

export const useCreateGrant = (
  resourceType: SharedResourceType,
  resourceId: string,
) => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();
  return useMutation({
    mutationFn: ({
      principalUserId,
      accessLevel,
    }: {
      principalUserId: string;
      accessLevel: GrantAccessLevel;
    }) =>
      grantApi.createGrant(tenantId, {
        resource_type: resourceType,
        resource_id: resourceId,
        principal_user_id: principalUserId,
        access_level: accessLevel,
      }),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: grantKeys.sharing(tenantId, resourceType, resourceId),
      });
    },
  });
};

export const useRevokeGrant = (
  resourceType: SharedResourceType,
  resourceId: string,
) => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();
  return useMutation({
    mutationFn: (grantId: string) => grantApi.revokeGrant(tenantId, grantId),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: grantKeys.sharing(tenantId, resourceType, resourceId),
      });
    },
  });
};
