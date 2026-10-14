import { useCallback, useState } from "react";
import {
  BannerSlim,
  Button,
  Flex,
  Input,
  Select,
  Text,
} from "src/components/design-system";
import { RoleDto } from "src/api/member/queries";
import { InvitationDto, useCreateInvitation } from "src/api/invitation/queries";

export function InviteMemberForm({ roles }: { roles: RoleDto[] }) {
  const [email, setEmail] = useState("");
  const [roleName, setRoleName] = useState("");
  const [lastInvitedEmail, setLastInvitedEmail] = useState<string | null>(null);
  const createInvitation = useCreateInvitation();
  const { mutate: invite } = createInvitation;

  const handleEmailChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) =>
      setEmail(event.target.value),
    [],
  );
  const handleInvited = useCallback((invitation: InvitationDto) => {
    setLastInvitedEmail(invitation.email);
    setEmail("");
  }, []);
  const handleInvite = useCallback(
    () => invite({ email, role_name: roleName }, { onSuccess: handleInvited }),
    [invite, email, roleName, handleInvited],
  );

  return (
    <Flex direction="column" gap="sm">
      <Text fontSize="sm" color="darkGrey">
        Invite people
      </Text>
      {createInvitation.error && (
        <BannerSlim type="error" message={createInvitation.error.message} />
      )}
      {lastInvitedEmail && (
        <BannerSlim
          type="success"
          message={`Invited ${lastInvitedEmail}. Ask them to sign in at ${window.location.origin} with that email address to accept.`}
        />
      )}
      <Flex direction="row" alignItems="center" gap="sm">
        <Input
          value={email}
          onChange={handleEmailChange}
          placeholder="Email address"
          type="email"
        />
        <Select value={roleName} onValueChange={setRoleName}>
          <Select.Trigger placeholder="Role" />
          <Select.Content>
            {roles
              .filter((role) => role.assignable)
              .map((role) => (
                <Select.Item key={role.name} value={role.name}>
                  {role.name}
                </Select.Item>
              ))}
          </Select.Content>
        </Select>
        <Button
          label="Invite"
          shape="rectangle"
          size="sm"
          backgroundColor="accent"
          onClick={handleInvite}
          disabled={!email.trim() || !roleName || createInvitation.isPending}
        />
      </Flex>
    </Flex>
  );
}
