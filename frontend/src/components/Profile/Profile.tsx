import { useCallback } from "react";
import {
  Avatar,
  BannerSlim,
  Button,
  Flex,
  Heading,
  Table,
  Text,
} from "src/components/design-system";
import { useDeleteSession, useSignedInUser } from "src/api/auth/queries";
import { SignInMethods } from "src/components/Profile/SignInMethods";
import {
  ACCOUNT_TABLE_COLUMNS,
  PROFILE_PAGE_STYLE,
} from "src/components/Profile/constants";

function AccountDetail({ label, value }: { label: string; value: string }) {
  return (
    <Table.Row aria-label={label}>
      <Table.Cell>
        <Text fontSize="sm" color="darkGrey">
          {label}
        </Text>
      </Table.Cell>
      <Table.Cell>
        <Text fontSize="sm" truncate>
          {value}
        </Text>
      </Table.Cell>
    </Table.Row>
  );
}

export function Profile() {
  const user = useSignedInUser();
  const signOut = useDeleteSession();
  const { mutate: deleteSession } = signOut;
  const handleSignOut = useCallback(() => deleteSession(), [deleteSession]);

  return (
    <Flex
      flex="grow"
      direction="column"
      paddingX="2xl"
      paddingY="lg"
      overflow="scrollY"
    >
      <Flex direction="column" gap="xl" width="100%" sx={PROFILE_PAGE_STYLE}>
        <Flex
          direction="row"
          justifyContent="space-between"
          alignItems="flex-start"
          gap="md"
        >
          <Flex direction="column" gap="2xs">
            <Heading accessibilityLevel={2} size="2xl" textColor="black">
              Profile
            </Heading>
            <Text fontSize="sm" color="subtle">
              Your account details and how you sign in.
            </Text>
          </Flex>
          <Button
            label="Sign out"
            leadingIconName="logout"
            shape="rectangle"
            size="sm"
            backgroundColor="white"
            onClick={handleSignOut}
            disabled={signOut.isPending}
          />
        </Flex>
        {signOut.error && (
          <BannerSlim type="error" message={signOut.error.message} />
        )}
        <Flex direction="row" alignItems="center" gap="sm">
          <Avatar name={user.full_name} />
          <Flex direction="column" overflow="hidden">
            <Text fontSize="sm" truncate>
              {user.full_name}
            </Text>
            <Text fontSize="xs" color="subtle" truncate>
              {user.email}
            </Text>
          </Flex>
        </Flex>
        <Flex direction="column" gap="sm">
          <Heading accessibilityLevel={3} size="lg">
            Account
          </Heading>
          <Table columns={ACCOUNT_TABLE_COLUMNS} aria-label="Account">
            <AccountDetail label="Full name" value={user.full_name} />
            <AccountDetail label="Email" value={user.email} />
            <AccountDetail label="Username" value={user.username} />
            <AccountDetail label="User ID" value={user.id} />
          </Table>
        </Flex>
        <SignInMethods />
      </Flex>
    </Flex>
  );
}
