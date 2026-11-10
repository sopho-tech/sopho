import { useCallback } from "react";
import {
  Badge,
  BannerSlim,
  DropdownMenu,
  Flex,
  IconButton,
  Table,
  Text,
  ToolTip,
} from "src/components/design-system";
import {
  EMAIL_STATUS_BADGE_VARIANTS,
  EMAIL_STATUS_LABELS,
  INVITATION_ACTIONS_LABEL,
  INVITATIONS_TABLE_COLUMNS,
} from "src/components/Settings/Members/constants";
import {
  EmailDeliveryDto,
  InvitationDto,
  useResendInvitation,
  useRevokeInvitation,
} from "src/api/invitation/queries";

function EmailDeliveryBadge({
  emailDelivery,
}: {
  emailDelivery: EmailDeliveryDto;
}) {
  const badge = (
    <Badge variant={EMAIL_STATUS_BADGE_VARIANTS[emailDelivery.status]}>
      {EMAIL_STATUS_LABELS[emailDelivery.status]}
    </Badge>
  );
  return emailDelivery.last_error ? (
    <ToolTip messageElement={emailDelivery.last_error}>{badge}</ToolTip>
  ) : (
    badge
  );
}

type InvitationRowProps = {
  invitation: InvitationDto;
  canResend: boolean;
  onResend: (invitationId: string) => void;
  onRevoke: (invitationId: string) => void;
  isUpdating: boolean;
};

function InvitationRow({
  invitation,
  canResend,
  onResend,
  onRevoke,
  isUpdating,
}: InvitationRowProps) {
  const handleRevoke = useCallback(
    () => onRevoke(invitation.id),
    [invitation.id, onRevoke],
  );
  const handleResend = useCallback(
    () => onResend(invitation.id),
    [invitation.id, onResend],
  );

  return (
    <Table.Row aria-label={invitation.email}>
      <Table.Cell>
        <Text fontSize="sm" truncate>
          {invitation.email}
        </Text>
      </Table.Cell>
      <Table.Cell>
        <Text fontSize="sm" color="darkGrey">
          {invitation.role_display_name}
        </Text>
      </Table.Cell>
      <Table.Cell>
        <Text fontSize="sm" color="darkGrey" truncate>
          {invitation.invited_by_name}
        </Text>
      </Table.Cell>
      <Table.Cell>
        {invitation.email_delivery && (
          <EmailDeliveryBadge emailDelivery={invitation.email_delivery} />
        )}
      </Table.Cell>
      <Table.Cell>
        <Badge variant={invitation.status === "EXPIRED" ? "yellow" : "subtle"}>
          {invitation.expiry_label}
        </Badge>
      </Table.Cell>
      <Table.Cell align="end">
        <DropdownMenu modal={false}>
          <DropdownMenu.Trigger>
            <IconButton
              type="more_horiz"
              backgroundColor="default"
              iconColor="grey"
              iconSize="md"
              disabled={isUpdating}
              aria-label={`${INVITATION_ACTIONS_LABEL} for ${invitation.email}`}
            />
          </DropdownMenu.Trigger>
          <DropdownMenu.Content>
            {canResend && (
              <DropdownMenu.Item icon="email" onClick={handleResend}>
                Resend invitation
              </DropdownMenu.Item>
            )}
            <DropdownMenu.Item icon="circle_x" onClick={handleRevoke}>
              Revoke invitation
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu>
      </Table.Cell>
    </Table.Row>
  );
}

type InvitationsTableProps = {
  invitations: InvitationDto[];
  canResend: boolean;
};

export function InvitationsTable({
  invitations,
  canResend,
}: InvitationsTableProps) {
  const revokeInvitation = useRevokeInvitation();
  const resendInvitation = useResendInvitation();
  const { mutate: revoke } = revokeInvitation;
  const { mutate: resend } = resendInvitation;
  const failure = revokeInvitation.error ?? resendInvitation.error;

  return (
    <Flex direction="column" gap="md">
      {failure && <BannerSlim type="error" message={failure.message} />}
      {invitations.length === 0 ? (
        <Text fontSize="sm" color="subtle">
          No pending invitations.
        </Text>
      ) : (
        <Table columns={INVITATIONS_TABLE_COLUMNS} aria-label="Invitations">
          <Table.Header>
            <Table.HeaderCell>Email</Table.HeaderCell>
            <Table.HeaderCell>Role</Table.HeaderCell>
            <Table.HeaderCell>Invited by</Table.HeaderCell>
            <Table.HeaderCell>Delivery</Table.HeaderCell>
            <Table.HeaderCell>Expiry</Table.HeaderCell>
            <Table.HeaderCell />
          </Table.Header>
          {invitations.map((invitation) => (
            <InvitationRow
              key={invitation.id}
              invitation={invitation}
              canResend={canResend}
              onResend={resend}
              onRevoke={revoke}
              isUpdating={
                revokeInvitation.isPending || resendInvitation.isPending
              }
            />
          ))}
        </Table>
      )}
    </Flex>
  );
}
