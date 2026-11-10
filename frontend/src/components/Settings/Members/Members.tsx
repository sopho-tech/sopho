import { useCallback, useState } from "react";
import {
  BannerSlim,
  Button,
  Flex,
  Heading,
  SegmentedControl,
  Text,
} from "src/components/design-system";
import {
  MemberDto,
  useRemoveMember,
  useRoles,
  useTenantMembers,
  useUpdateMemberRole,
} from "src/api/member/queries";
import { useTenantPermission } from "src/api/tenant/queries";
import {
  InvitationDeliveryMethod,
  InvitationDto,
  useInvitationDelivery,
  useTenantInvitations,
} from "src/api/invitation/queries";
import { PERMISSIONS } from "src/constants/permissions";
import { InviteMemberDialog } from "src/components/Settings/Members/InviteMemberDialog";
import { InvitationsTable } from "src/components/Settings/Members/InvitationsTable";
import { MembersTable } from "src/components/Settings/Members/MembersTable";
import { RemoveMemberDialog } from "src/components/Settings/Members/RemoveMemberDialog";
import {
  INVITATIONS_TAB,
  MEMBERS_TAB,
} from "src/components/Settings/Members/constants";

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

export function Members() {
  const members = useTenantMembers();
  const roles = useRoles();
  const canInvite = useTenantPermission(PERMISSIONS.MEMBER_INVITE);
  const invitationDelivery = useInvitationDelivery(canInvite);
  const invitations = useTenantInvitations(canInvite);
  const updateMemberRole = useUpdateMemberRole();
  const removeMember = useRemoveMember();
  const [selectedTab, setSelectedTab] = useState(MEMBERS_TAB);
  const [isInviting, setIsInviting] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [memberPendingRemoval, setMemberPendingRemoval] =
    useState<MemberDto | null>(null);

  const { mutate: updateRole } = updateMemberRole;
  const { mutate: remove } = removeMember;
  const deliveryMethod = invitationDelivery.data?.method;
  const canSendInvitations =
    deliveryMethod !== undefined && deliveryMethod !== "UNAVAILABLE";

  const handleChangeRole = useCallback(
    (userId: string, roleName: string) => updateRole({ userId, roleName }),
    [updateRole],
  );
  const handleOpenInvite = useCallback(() => {
    setNotice(null);
    setIsInviting(true);
  }, []);
  const handleCloseInvite = useCallback(() => setIsInviting(false), []);
  const handleInvited = useCallback(
    (invitation: InvitationDto) => {
      if (!deliveryMethod) {
        throw new Error("Invitation delivery is not loaded");
      }
      setIsInviting(false);
      setNotice(invitedMessage(deliveryMethod, invitation.email));
      setSelectedTab(INVITATIONS_TAB);
    },
    [deliveryMethod],
  );
  const handleCloseRemoval = useCallback(
    () => setMemberPendingRemoval(null),
    [],
  );
  const handleConfirmRemoval = useCallback(() => {
    if (!memberPendingRemoval) {
      throw new Error("No member is pending removal");
    }
    remove(memberPendingRemoval.user_id, {
      onSuccess: () => {
        setNotice(
          `${memberPendingRemoval.full_name} is removed from the tenant.`,
        );
        setMemberPendingRemoval(null);
      },
    });
  }, [memberPendingRemoval, remove]);

  const failure =
    members.error ??
    roles.error ??
    invitationDelivery.error ??
    invitations.error ??
    updateMemberRole.error ??
    removeMember.error;

  return (
    <Flex direction="column" gap="lg">
      <Flex
        direction="row"
        justifyContent="space-between"
        alignItems="flex-start"
        gap="md"
      >
        <Flex direction="column" gap="2xs">
          <Heading accessibilityLevel={2} size="2xl" textColor="black">
            Tenant Management
          </Heading>
          <Text fontSize="sm" color="subtle">
            Manage who has access to this tenant and what they can do.
          </Text>
        </Flex>
        {canSendInvitations && roles.data && (
          <Button
            label="Invite people"
            leadingIconName="add"
            shape="rectangle"
            size="sm"
            backgroundColor="accent"
            onClick={handleOpenInvite}
          />
        )}
      </Flex>
      {failure && <BannerSlim type="error" message={failure.message} />}
      {invitationDelivery.data?.unavailable_reason && (
        <BannerSlim
          type="info"
          message={invitationDelivery.data.unavailable_reason}
        />
      )}
      {notice && <BannerSlim type="success" message={notice} />}
      {canInvite && members.data && invitations.data && (
        <Flex direction="row">
          <SegmentedControl
            size="sm"
            value={selectedTab}
            onValueChange={setSelectedTab}
            options={[
              {
                value: MEMBERS_TAB,
                label: `Members (${members.data.length})`,
              },
              {
                value: INVITATIONS_TAB,
                label: `Invitations (${invitations.data.length})`,
              },
            ]}
          />
        </Flex>
      )}
      {selectedTab === MEMBERS_TAB && members.data && roles.data && (
        <MembersTable
          members={members.data}
          roles={roles.data}
          onChangeRole={handleChangeRole}
          onRemove={setMemberPendingRemoval}
        />
      )}
      {selectedTab === INVITATIONS_TAB && invitations.data && (
        <InvitationsTable
          invitations={invitations.data}
          canResend={deliveryMethod === "EMAIL"}
        />
      )}
      {isInviting && roles.data && (
        <InviteMemberDialog
          roles={roles.data}
          onInvited={handleInvited}
          onClose={handleCloseInvite}
        />
      )}
      {memberPendingRemoval && (
        <RemoveMemberDialog
          member={memberPendingRemoval}
          onConfirm={handleConfirmRemoval}
          onClose={handleCloseRemoval}
          isPending={removeMember.isPending}
        />
      )}
    </Flex>
  );
}
