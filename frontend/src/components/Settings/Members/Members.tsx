import { useCallback, useState } from "react";
import { useNavigate } from "react-router";
import { BannerSlim, Flex, Heading, Text } from "src/components/design-system";
import {
  MemberDto,
  useRemoveMember,
  useRoles,
  useTenantMembers,
  useUpdateMemberRole,
} from "src/api/member/queries";
import { APP_ROUTES } from "src/constants/app_routes";
import { InvitationsLocationState } from "src/components/Settings/Members/Invitations";
import { InvitePeopleButton } from "src/components/Settings/Members/InvitePeopleButton";
import { MembersTable } from "src/components/Settings/Members/MembersTable";
import { RemoveMemberDialog } from "src/components/Settings/Members/RemoveMemberDialog";
import { useTenantPath } from "src/utils/tenant";

export function Members() {
  const navigate = useNavigate();
  const toTenantPath = useTenantPath();
  const members = useTenantMembers();
  const roles = useRoles();
  const updateMemberRole = useUpdateMemberRole();
  const removeMember = useRemoveMember();
  const [notice, setNotice] = useState<string | null>(null);
  const [memberPendingRemoval, setMemberPendingRemoval] =
    useState<MemberDto | null>(null);

  const { mutate: updateRole } = updateMemberRole;
  const { mutate: remove } = removeMember;

  const handleChangeRole = useCallback(
    (userId: string, roleName: string) => updateRole({ userId, roleName }),
    [updateRole],
  );
  const handleInvited = useCallback(
    (invitedNotice: string) => {
      const invitationsLocationState: InvitationsLocationState = {
        notice: invitedNotice,
      };
      void navigate(toTenantPath(APP_ROUTES.SETTINGS_ROUTES.INVITATIONS), {
        state: invitationsLocationState,
      });
    },
    [navigate, toTenantPath],
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
            Members
          </Heading>
          <Text fontSize="sm" color="subtle">
            Manage who has access to this tenant and what they can do.
          </Text>
        </Flex>
        <InvitePeopleButton onInvited={handleInvited} />
      </Flex>
      {failure && <BannerSlim type="error" message={failure.message} />}
      {notice && <BannerSlim type="success" message={notice} />}
      {members.data && roles.data && (
        <MembersTable
          members={members.data}
          roles={roles.data}
          onChangeRole={handleChangeRole}
          onRemove={setMemberPendingRemoval}
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
