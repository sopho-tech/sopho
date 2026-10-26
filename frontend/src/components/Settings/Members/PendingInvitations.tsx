import { useCallback } from "react";
import { BannerSlim, Button, Flex, Text } from "src/components/design-system";
import {
  EMAIL_STATUS_LABELS,
  UNSTYLED_LIST_STYLE,
} from "src/components/Settings/Members/constants";
import {
  EmailDeliveryDto,
  InvitationDeliveryMethod,
  InvitationDto,
  useResendInvitation,
  useRevokeInvitation,
  useTenantInvitations,
} from "src/api/invitation/queries";

function emailDeliveryLabel(emailDelivery: EmailDeliveryDto): string {
  const statusLabel = EMAIL_STATUS_LABELS[emailDelivery.status];
  return emailDelivery.last_error && emailDelivery.status === "FAILED"
    ? `${statusLabel}: ${emailDelivery.last_error}`
    : statusLabel;
}

function PendingInvitationRow({
  invitation,
  canResend,
  onResend,
  onRevoke,
  isUpdating,
}: {
  invitation: InvitationDto;
  canResend: boolean;
  onResend: (invitationId: string) => void;
  onRevoke: (invitationId: string) => void;
  isUpdating: boolean;
}) {
  const handleRevoke = useCallback(
    () => onRevoke(invitation.id),
    [invitation.id, onRevoke],
  );
  const handleResend = useCallback(
    () => onResend(invitation.id),
    [invitation.id, onResend],
  );
  return (
    <Flex
      as="li"
      direction="row"
      justifyContent="space-between"
      alignItems="center"
      gap="md"
      aria-label={invitation.email}
    >
      <Flex direction="column">
        <Text fontSize="sm">{invitation.email}</Text>
        <Text fontSize="xs" color="subtle">
          {`${invitation.role_name} · invited by ${invitation.invited_by_name}`}
        </Text>
        {invitation.email_delivery && (
          <Text
            fontSize="xs"
            color={
              invitation.email_delivery.status === "FAILED" ? "error" : "subtle"
            }
          >
            {emailDeliveryLabel(invitation.email_delivery)}
          </Text>
        )}
      </Flex>
      <Flex direction="row" alignItems="center" gap="sm">
        <Text fontSize="xs" color="subtle">
          {invitation.status === "EXPIRED"
            ? "Expired"
            : `Expires ${new Date(invitation.expires_at).toLocaleDateString()}`}
        </Text>
        {canResend && (
          <Button
            label="Resend"
            shape="rectangle"
            size="sm"
            backgroundColor="white"
            onClick={handleResend}
            disabled={isUpdating}
          />
        )}
        <Button
          label="Revoke"
          shape="rectangle"
          size="sm"
          backgroundColor="white"
          onClick={handleRevoke}
          disabled={isUpdating}
        />
      </Flex>
    </Flex>
  );
}

export function PendingInvitations({
  deliveryMethod,
}: {
  deliveryMethod: InvitationDeliveryMethod;
}) {
  const invitations = useTenantInvitations(true);
  const revokeInvitation = useRevokeInvitation();
  const resendInvitation = useResendInvitation();
  const { mutate: revoke } = revokeInvitation;
  const { mutate: resend } = resendInvitation;

  if (invitations.error) {
    return <BannerSlim type="error" message={invitations.error.message} />;
  }
  if (!invitations.data || invitations.data.length === 0) {
    return null;
  }

  return (
    <Flex direction="column" gap="sm">
      <Text fontSize="sm" color="darkGrey">
        Pending invitations
      </Text>
      {revokeInvitation.error && (
        <BannerSlim type="error" message={revokeInvitation.error.message} />
      )}
      {resendInvitation.error && (
        <BannerSlim type="error" message={resendInvitation.error.message} />
      )}
      <Flex as="ul" direction="column" gap="sm" sx={UNSTYLED_LIST_STYLE}>
        {invitations.data.map((invitation) => (
          <PendingInvitationRow
            key={invitation.id}
            invitation={invitation}
            canResend={deliveryMethod === "EMAIL"}
            onResend={resend}
            onRevoke={revoke}
            isUpdating={
              revokeInvitation.isPending || resendInvitation.isPending
            }
          />
        ))}
      </Flex>
    </Flex>
  );
}
