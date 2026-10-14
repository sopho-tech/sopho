import { useCallback, useState } from "react";
import { Dialog } from "src/components/Dialog/Dialog";
import {
  BannerSlim,
  Button,
  Flex,
  IconButton,
  SegmentedControl,
  Select,
  Text,
} from "src/components/design-system";
import {
  GrantAccessLevel,
  GrantDto,
  ResourceVisibility,
  SharedResourceType,
  useCreateGrant,
  useResourceSharing,
  useRevokeGrant,
  useUpdateVisibility,
} from "src/api/grant/queries";
import { TenantMemberDto, useTenantMembers } from "src/api/tenant/queries";
import {
  ACCESS_LEVEL_OPTIONS,
  DEFAULT_ACCESS_LEVEL,
  REMOVE_ACCESS_TOOLTIP,
  VISIBILITY_OPTIONS,
} from "src/components/ShareDialog/constants";

type ShareDialogProps = {
  resourceType: SharedResourceType;
  resourceId: string;
  resourceName: string;
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
};

type GrantRowProps = {
  grant: GrantDto;
  onChangeAccessLevel: (
    principalUserId: string,
    accessLevel: GrantAccessLevel,
  ) => void;
  onRevoke: (grantId: string) => void;
};

function MemberLabel({ member }: { member: TenantMemberDto }) {
  return (
    <Flex direction="column">
      <Text fontSize="sm">{member.full_name}</Text>
      <Text fontSize="xs" color="subtle">
        {member.email}
      </Text>
    </Flex>
  );
}

function AccessLevelSelect({
  value,
  onValueChange,
  disabled,
}: {
  value: GrantAccessLevel;
  onValueChange: (accessLevel: string) => void;
  disabled?: boolean;
}) {
  return (
    <Select value={value} onValueChange={onValueChange} disabled={disabled}>
      <Select.Trigger />
      <Select.Content>
        {ACCESS_LEVEL_OPTIONS.map((option) => (
          <Select.Item key={option.value} value={option.value}>
            {option.label}
          </Select.Item>
        ))}
      </Select.Content>
    </Select>
  );
}

function GrantRow({ grant, onChangeAccessLevel, onRevoke }: GrantRowProps) {
  const handleAccessLevelChange = useCallback(
    (accessLevel: string) =>
      onChangeAccessLevel(
        grant.principal_user_id,
        accessLevel as GrantAccessLevel,
      ),
    [grant.principal_user_id, onChangeAccessLevel],
  );
  const handleRevoke = useCallback(
    () => onRevoke(grant.id),
    [grant.id, onRevoke],
  );

  return (
    <Flex
      direction="row"
      justifyContent="space-between"
      alignItems="center"
      gap="sm"
    >
      {grant.principal ? (
        <MemberLabel member={grant.principal} />
      ) : (
        <Text fontSize="sm" color="subtle">
          No longer a member of this workspace
        </Text>
      )}
      <Flex direction="row" alignItems="center" gap="xs">
        <AccessLevelSelect
          value={grant.access_level}
          onValueChange={handleAccessLevelChange}
          disabled={!grant.principal}
        />
        <IconButton
          type="delete"
          backgroundColor="default"
          iconColor="black"
          iconSize="md"
          onClick={handleRevoke}
          tooltip={REMOVE_ACCESS_TOOLTIP}
        />
      </Flex>
    </Flex>
  );
}

export function ShareDialog({
  resourceType,
  resourceId,
  resourceName,
  isOpen,
  onOpenChange,
}: ShareDialogProps) {
  const sharing = useResourceSharing(resourceType, resourceId, isOpen);
  const members = useTenantMembers();
  const updateVisibility = useUpdateVisibility(resourceType, resourceId);
  const createGrant = useCreateGrant(resourceType, resourceId);
  const revokeGrant = useRevokeGrant(resourceType, resourceId);
  const [selectedMemberId, setSelectedMemberId] = useState("");
  const [selectedAccessLevel, setSelectedAccessLevel] =
    useState<GrantAccessLevel>(DEFAULT_ACCESS_LEVEL);

  const failure =
    sharing.error ??
    members.error ??
    updateVisibility.error ??
    createGrant.error ??
    revokeGrant.error;

  const grantedUserIds = new Set(
    sharing.data?.grants.map((grant) => grant.principal_user_id),
  );
  const shareableMembers = (members.data ?? []).filter(
    (member) =>
      member.user_id !== sharing.data?.owner?.user_id &&
      !grantedUserIds.has(member.user_id),
  );

  const { mutate: mutateVisibility } = updateVisibility;
  const { mutate: mutateGrant } = createGrant;
  const { mutate: mutateRevoke } = revokeGrant;

  const handleVisibilityChange = useCallback(
    (visibility: string) => mutateVisibility(visibility as ResourceVisibility),
    [mutateVisibility],
  );
  const handleChangeAccessLevel = useCallback(
    (principalUserId: string, accessLevel: GrantAccessLevel) =>
      mutateGrant({ principalUserId, accessLevel }),
    [mutateGrant],
  );
  const handleRevoke = useCallback(
    (grantId: string) => mutateRevoke(grantId),
    [mutateRevoke],
  );
  const handleSelectedAccessLevelChange = useCallback(
    (accessLevel: string) =>
      setSelectedAccessLevel(accessLevel as GrantAccessLevel),
    [],
  );
  const handleShare = useCallback(
    () =>
      mutateGrant(
        { principalUserId: selectedMemberId, accessLevel: selectedAccessLevel },
        { onSuccess: () => setSelectedMemberId("") },
      ),
    [mutateGrant, selectedMemberId, selectedAccessLevel],
  );
  const handleDialogClose = useCallback(
    () => onOpenChange(false),
    [onOpenChange],
  );

  const content = (
    <Flex direction="column" gap="lg">
      {failure && <BannerSlim type="error" message={failure.message} />}

      <Flex direction="column" gap="sm">
        <Text fontSize="sm" color="darkGrey">
          General access
        </Text>
        <SegmentedControl
          size="sm"
          options={VISIBILITY_OPTIONS}
          value={sharing.data?.visibility ?? ""}
          onValueChange={handleVisibilityChange}
          disabled={!sharing.data || updateVisibility.isPending}
        />
      </Flex>

      <Flex direction="column" gap="sm">
        <Text fontSize="sm" color="darkGrey">
          People with access
        </Text>
        {sharing.data?.owner && (
          <Flex
            direction="row"
            justifyContent="space-between"
            alignItems="center"
          >
            <MemberLabel member={sharing.data.owner} />
            <Text fontSize="sm" color="subtle">
              Owner
            </Text>
          </Flex>
        )}
        {sharing.data?.grants.map((grant) => (
          <GrantRow
            key={grant.id}
            grant={grant}
            onChangeAccessLevel={handleChangeAccessLevel}
            onRevoke={handleRevoke}
          />
        ))}
      </Flex>

      <Flex direction="column" gap="sm">
        <Text fontSize="sm" color="darkGrey">
          Add people
        </Text>
        {shareableMembers.length === 0 ? (
          <Text fontSize="sm" color="subtle">
            Everyone in this workspace already has access
          </Text>
        ) : (
          <Flex direction="row" alignItems="center" gap="sm">
            <Select
              value={selectedMemberId}
              onValueChange={setSelectedMemberId}
            >
              <Select.Trigger placeholder="Choose a person" />
              <Select.Content>
                {shareableMembers.map((member) => (
                  <Select.Item
                    key={member.user_id}
                    value={member.user_id}
                    textValue={member.full_name}
                  >
                    {member.full_name}
                  </Select.Item>
                ))}
              </Select.Content>
            </Select>
            <AccessLevelSelect
              value={selectedAccessLevel}
              onValueChange={handleSelectedAccessLevelChange}
            />
            <Button
              label="Share"
              shape="rectangle"
              size="sm"
              backgroundColor="accent"
              onClick={handleShare}
              disabled={!selectedMemberId || createGrant.isPending}
            />
          </Flex>
        )}
      </Flex>
    </Flex>
  );

  return (
    <Dialog
      shouldOpenDialog={isOpen}
      handleOnOpenChange={onOpenChange}
      handleDialogClose={handleDialogClose}
      info={content}
      title={`Share "${resourceName}"`}
    />
  );
}
