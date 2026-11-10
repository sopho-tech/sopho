import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { tenantApiUrl, useTenantId } from "src/utils/tenant";
import { tenantKeys } from "src/api/tenant/queries";

export type InvitationStatus = "PENDING" | "EXPIRED";

export type EmailStatus =
  "PENDING" | "SENDING" | "SENT" | "FAILED" | "CANCELLED";

export type EmailDeliveryDto = {
  status: EmailStatus;
  last_error: string | null;
};

export type InvitationDeliveryMethod =
  "EMAIL" | "SINGLE_SIGN_ON" | "UNAVAILABLE";

export type InvitationDeliveryDto = {
  method: InvitationDeliveryMethod;
  unavailable_reason: string | null;
};

export type InvitationDto = {
  id: string;
  email: string;
  role_name: string;
  role_display_name: string;
  status: InvitationStatus;
  invited_by_name: string;
  expires_at: string;
  expiry_label: string;
  email_delivery: EmailDeliveryDto | null;
};

export type ReceivedInvitationDto = {
  id: string;
  tenant_id: string;
  tenant_name: string;
  role_name: string;
  role_display_name: string;
  invited_by_name: string;
  expires_at: string;
};

export type AcceptedInvitationDto = {
  tenant_id: string;
};

export type CreateInvitationDto = {
  email: string;
  role_name: string;
};

export const invitationKeys = {
  received: ["invitations", "received"] as const,
  tenant: (tenantId: string) => [tenantId, "invitations"] as const,
  delivery: (tenantId: string) => [tenantId, "invitationDelivery"] as const,
};

const invitationApi = {
  listReceived: async (): Promise<ReceivedInvitationDto[]> => {
    const response = await ApiService.get({
      url: API_ENDPOINTS.INVITATION.RECEIVED,
      onlyBody: true,
    });
    return response as ReceivedInvitationDto[];
  },

  accept: async (invitationId: string): Promise<AcceptedInvitationDto> => {
    const response = await ApiService.post({
      url: API_ENDPOINTS.INVITATION.ACCEPT.replace(
        ":invitation_id",
        invitationId,
      ),
    });
    return response as AcceptedInvitationDto;
  },

  listForTenant: async (tenantId: string): Promise<InvitationDto[]> => {
    const response = await ApiService.get({
      url: tenantApiUrl(tenantId, API_ENDPOINTS.INVITATION.TENANT),
      onlyBody: true,
    });
    return response as InvitationDto[];
  },

  create: async (
    tenantId: string,
    payload: CreateInvitationDto,
  ): Promise<InvitationDto> => {
    const response = await ApiService.post({
      url: tenantApiUrl(tenantId, API_ENDPOINTS.INVITATION.TENANT),
      data: payload,
      headers: { "Content-Type": "application/json" },
    });
    return response as InvitationDto;
  },

  getDelivery: async (tenantId: string): Promise<InvitationDeliveryDto> => {
    const response = await ApiService.get({
      url: tenantApiUrl(tenantId, API_ENDPOINTS.INVITATION.DELIVERY),
      onlyBody: true,
    });
    return response as InvitationDeliveryDto;
  },

  resend: async (
    tenantId: string,
    invitationId: string,
  ): Promise<InvitationDto> => {
    const response = await ApiService.post({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.INVITATION.RESEND.replace(":invitation_id", invitationId),
      ),
    });
    return response as InvitationDto;
  },

  revoke: async (tenantId: string, invitationId: string): Promise<void> => {
    await ApiService.delete({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.INVITATION.REVOKE.replace(":invitation_id", invitationId),
      ),
    });
  },
};

export const useReceivedInvitations = () => {
  return useQuery({
    queryKey: invitationKeys.received,
    queryFn: invitationApi.listReceived,
  });
};

export const useAcceptInvitation = (callbacks: {
  onSuccess: (accepted: AcceptedInvitationDto) => void;
}) => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: invitationApi.accept,
    onSuccess: async (accepted) => {
      await queryClient.invalidateQueries({ queryKey: tenantKeys.memberships });
      await queryClient.invalidateQueries({
        queryKey: invitationKeys.received,
      });
      callbacks.onSuccess(accepted);
    },
  });
};

export const useTenantInvitations = (enabled: boolean) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: invitationKeys.tenant(tenantId),
    queryFn: () => invitationApi.listForTenant(tenantId),
    enabled,
  });
};

export const useCreateInvitation = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();
  return useMutation({
    mutationFn: (payload: CreateInvitationDto) =>
      invitationApi.create(tenantId, payload),
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: invitationKeys.tenant(tenantId),
      }),
  });
};

export const useRevokeInvitation = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();
  return useMutation({
    mutationFn: (invitationId: string) =>
      invitationApi.revoke(tenantId, invitationId),
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: invitationKeys.tenant(tenantId),
      }),
  });
};

export const useInvitationDelivery = (enabled: boolean) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: invitationKeys.delivery(tenantId),
    queryFn: () => invitationApi.getDelivery(tenantId),
    enabled,
  });
};

export const useResendInvitation = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();
  return useMutation({
    mutationFn: (invitationId: string) =>
      invitationApi.resend(tenantId, invitationId),
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: invitationKeys.tenant(tenantId),
      }),
  });
};
