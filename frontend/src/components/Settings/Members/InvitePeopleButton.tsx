import { useCallback, useState } from "react";
import { Button } from "src/components/design-system";
import { useRoles } from "src/api/member/queries";
import { useTenantPermission } from "src/api/tenant/queries";
import {
  InvitationDeliveryMethod,
  InvitationDto,
  useInvitationDelivery,
} from "src/api/invitation/queries";
import { PERMISSIONS } from "src/constants/permissions";
import { InviteMemberDialog } from "src/components/Settings/Members/InviteMemberDialog";

function invitedMessage(
  deliveryMethod: InvitationDeliveryMethod,
  email: string,
): string {
  switch (deliveryMethod) {
    case "EMAIL":
      return `Invited ${email}. Sopho is emailing them a link to join.`;
    case "SINGLE_SIGN_ON":
      return `Invited ${email}. Ask them to sign in at ${window.location.origin} with that email address to accept.`;
    case "UNAVAILABLE":
      throw new Error("Invitations are unavailable, so nobody was invited");
  }
}

type InvitePeopleButtonProps = {
  onInvited: (notice: string) => void;
};

export function InvitePeopleButton({ onInvited }: InvitePeopleButtonProps) {
  const roles = useRoles();
  const canInvite = useTenantPermission(PERMISSIONS.MEMBER_INVITE);
  const invitationDelivery = useInvitationDelivery(canInvite);
  const [isInviting, setIsInviting] = useState(false);
  const deliveryMethod = invitationDelivery.data?.method;

  const handleOpenInvite = useCallback(() => setIsInviting(true), []);
  const handleCloseInvite = useCallback(() => setIsInviting(false), []);
  const handleInvited = useCallback(
    (invitation: InvitationDto) => {
      if (!deliveryMethod) {
        throw new Error("Invitation delivery is not loaded");
      }
      setIsInviting(false);
      onInvited(invitedMessage(deliveryMethod, invitation.email));
    },
    [deliveryMethod, onInvited],
  );

  if (!deliveryMethod || deliveryMethod === "UNAVAILABLE" || !roles.data) {
    return null;
  }

  return (
    <>
      <Button
        label="Invite"
        leadingIconName="add"
        shape="rectangle"
        size="sm"
        backgroundColor="accent"
        onClick={handleOpenInvite}
      />
      {isInviting && (
        <InviteMemberDialog
          roles={roles.data}
          onInvited={handleInvited}
          onClose={handleCloseInvite}
        />
      )}
    </>
  );
}
