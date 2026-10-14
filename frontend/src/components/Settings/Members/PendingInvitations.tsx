import { useCallback } from "react";
import { BannerSlim, Button, Flex, Text } from "src/components/design-system";
import {
  InvitationDto,
  useRevokeInvitation,
  useTenantInvitations,
} from "src/api/invitation/queries";

function PendingInvitationRow({
  invitation,
  onRevoke,
  isRevoking,
}: {
  invitation: InvitationDto;
  onRevoke: (invitationId: string) => void;
  isRevoking: boolean;
}) {
  const handleRevoke = useCallback(
    () => onRevoke(invitation.id),
    [invitation.id, onRevoke],
  );
  return (
    <Flex
      direction="row"
      justifyContent="space-between"
      alignItems="center"
      gap="md"
      role="group"
      aria-label={invitation.email}
    >
      <Flex direction="column">
        <Text fontSize="sm">{invitation.email}</Text>
        <Text fontSize="xs" color="subtle">
          {`${invitation.role_name} · invited by ${invitation.invited_by_name}`}
        </Text>
      </Flex>
      <Flex direction="row" alignItems="center" gap="sm">
        <Text fontSize="xs" color="subtle">
          {invitation.status === "EXPIRED"
            ? "Expired"
            : `Expires ${new Date(invitation.expires_at).toLocaleDateString()}`}
        </Text>
        <Button
          label="Revoke"
          shape="rectangle"
          size="sm"
          backgroundColor="white"
          onClick={handleRevoke}
          disabled={isRevoking}
        />
      </Flex>
    </Flex>
  );
}

export function PendingInvitations() {
  const invitations = useTenantInvitations(true);
  const revokeInvitation = useRevokeInvitation();
  const { mutate: revoke } = revokeInvitation;

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
      {invitations.data.map((invitation) => (
        <PendingInvitationRow
          key={invitation.id}
          invitation={invitation}
          onRevoke={revoke}
          isRevoking={revokeInvitation.isPending}
        />
      ))}
    </Flex>
  );
}
