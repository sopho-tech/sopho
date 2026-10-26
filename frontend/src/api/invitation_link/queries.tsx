import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { authKeys } from "src/api/auth/queries";
import {
  AcceptedInvitationDto,
  invitationKeys,
} from "src/api/invitation/queries";
import { tenantKeys } from "src/api/tenant/queries";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { ApiService } from "src/utils/api_client";

export type InvitationLinkNextStep =
  "SIGN_UP" | "SIGN_IN" | "JOIN" | "WRONG_ACCOUNT";

export type InvitationLinkDto = {
  tenant_name: string;
  role_name: string;
  invited_by_name: string;
  email: string;
  expires_at: string;
  next_step: InvitationLinkNextStep;
};

export type InvitationSignUpDto = {
  token: string;
  full_name: string;
  password: string;
};

export const invitationLinkKeys = {
  lookup: (token: string) => ["invitationLink", token] as const,
};

const invitationLinkApi = {
  lookup: async (token: string): Promise<InvitationLinkDto> => {
    const response = await ApiService.post({
      url: API_ENDPOINTS.INVITATION_LINK.LOOKUP,
      data: { token },
      headers: { "Content-Type": "application/json" },
    });
    return response as InvitationLinkDto;
  },

  signUp: async (
    signUp: InvitationSignUpDto,
  ): Promise<AcceptedInvitationDto> => {
    const response = await ApiService.post({
      url: API_ENDPOINTS.INVITATION_LINK.SIGN_UP,
      data: signUp,
      headers: { "Content-Type": "application/json" },
    });
    return response as AcceptedInvitationDto;
  },

  accept: async (token: string): Promise<AcceptedInvitationDto> => {
    const response = await ApiService.post({
      url: API_ENDPOINTS.INVITATION_LINK.ACCEPT,
      data: { token },
      headers: { "Content-Type": "application/json" },
    });
    return response as AcceptedInvitationDto;
  },
};

export const useInvitationLink = (token: string) => {
  return useQuery({
    queryKey: invitationLinkKeys.lookup(token),
    queryFn: () => invitationLinkApi.lookup(token),
    staleTime: 0,
  });
};

export const useSignUpFromInvitation = (callbacks: {
  onSuccess: (accepted: AcceptedInvitationDto) => void;
}) => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: invitationLinkApi.signUp,
    onSuccess: async (accepted) => {
      await queryClient.invalidateQueries({ queryKey: authKeys.all });
      await queryClient.invalidateQueries({ queryKey: tenantKeys.memberships });
      callbacks.onSuccess(accepted);
    },
  });
};

export const useAcceptInvitationLink = (callbacks: {
  onSuccess: (accepted: AcceptedInvitationDto) => void;
}) => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: invitationLinkApi.accept,
    onSuccess: async (accepted) => {
      await queryClient.invalidateQueries({ queryKey: tenantKeys.memberships });
      await queryClient.invalidateQueries({
        queryKey: invitationKeys.received,
      });
      callbacks.onSuccess(accepted);
    },
  });
};
