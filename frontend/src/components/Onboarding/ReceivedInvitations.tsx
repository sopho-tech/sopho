import { useCallback } from "react";
import { useNavigate } from "react-router";
import { BannerSlim, Button, Flex, Text } from "src/components/design-system";
import {
  AcceptedInvitationDto,
  ReceivedInvitationDto,
  useAcceptInvitation,
} from "src/api/invitation/queries";
import { APP_ROUTES } from "src/constants/app_routes";
import { tenantPath } from "src/utils/tenant";

function InvitationRow({
  invitation,
  onAccept,
  isAccepting,
}: {
  invitation: ReceivedInvitationDto;
  onAccept: (invitationId: string) => void;
  isAccepting: boolean;
}) {
  const handleAccept = useCallback(
    () => onAccept(invitation.id),
    [invitation.id, onAccept],
  );
  return (
    <Flex
      direction="row"
      justifyContent="space-between"
      alignItems="center"
      gap="md"
    >
      <Flex direction="column">
        <Text fontSize="sm">{invitation.tenant_name}</Text>
        <Text fontSize="xs" color="subtle">
          {`Invited by ${invitation.invited_by_name} as ${invitation.role_name}`}
        </Text>
      </Flex>
      <Button
        label="Accept"
        shape="rectangle"
        size="sm"
        backgroundColor="accent"
        onClick={handleAccept}
        disabled={isAccepting}
      />
    </Flex>
  );
}

export function ReceivedInvitations({
  invitations,
}: {
  invitations: ReceivedInvitationDto[];
}) {
  const navigate = useNavigate();
  const handleAccepted = useCallback(
    (accepted: AcceptedInvitationDto) => {
      void navigate(tenantPath(accepted.tenant_id, APP_ROUTES.INDEX), {
        replace: true,
      });
    },
    [navigate],
  );
  const acceptInvitation = useAcceptInvitation({ onSuccess: handleAccepted });
  const { mutate: accept } = acceptInvitation;

  return (
    <Flex direction="column" gap="sm">
      <Text fontSize="sm" color="darkGrey">
        Invitations
      </Text>
      {acceptInvitation.error && (
        <BannerSlim type="error" message={acceptInvitation.error.message} />
      )}
      {invitations.map((invitation) => (
        <InvitationRow
          key={invitation.id}
          invitation={invitation}
          onAccept={accept}
          isAccepting={acceptInvitation.isPending}
        />
      ))}
    </Flex>
  );
}
