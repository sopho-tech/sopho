import { ChangeEvent, useCallback, useState } from "react";
import {
  Badge,
  BannerSlim,
  Box,
  DropdownMenu,
  Flex,
  Heading,
  IconButton,
  Input,
  Text,
} from "src/components/design-system";
import {
  PlatformTenantDto,
  PlatformUserDto,
  TenantSummaryDto,
  useDeactivateUser,
  usePlatformUsers,
  useReactivateUser,
  useSendPasswordReset,
} from "src/api/platform/queries";
import { AppointOwnerDialog } from "src/components/Platform/AppointOwnerDialog";
import { DeactivateUserDialog } from "src/components/Platform/DeactivateUserDialog";
import { PlatformPagination } from "src/components/Platform/PlatformPagination";
import {
  FIRST_PAGE,
  SEARCH_DEBOUNCE_MILLISECONDS,
  SEARCH_INPUT_STYLE,
  USER_ACTIONS_LABEL,
} from "src/components/Platform/constants";
import { UNSTYLED_LIST_STYLE } from "src/components/Settings/Members/constants";
import { useDebouncedValue } from "src/utils/hooks";

function signInMethodsLabel(user: PlatformUserDto): string {
  const methods = user.has_password
    ? ["password", ...user.provider_names]
    : user.provider_names;
  return methods.length > 0
    ? `Signs in with ${methods.join(", ")}`
    : "No sign-in method";
}

function tenantsLabel(user: PlatformUserDto): string {
  return user.tenants.length > 0
    ? user.tenants
        .map(
          (membership) => `${membership.tenant_name} (${membership.role_name})`,
        )
        .join(", ")
    : "Not in any tenant";
}

function tenantsWithoutOwnerMessage(tenants: TenantSummaryDto[]): string {
  const tenantNames = tenants.map((tenant) => tenant.tenant_name).join(", ");
  const verb = tenants.length === 1 ? "has" : "have";
  return `${tenantNames} ${verb} no active owner now. Use "Make owner of a tenant" on someone's row to appoint one.`;
}

type UserRowProps = {
  user: PlatformUserDto;
  onDeactivate: (user: PlatformUserDto) => void;
  onReactivate: (user: PlatformUserDto) => void;
  onSendPasswordReset: (user: PlatformUserDto) => void;
  onAppointOwner: (user: PlatformUserDto) => void;
  isUpdating: boolean;
};

function UserRow({
  user,
  onDeactivate,
  onReactivate,
  onSendPasswordReset,
  onAppointOwner,
  isUpdating,
}: UserRowProps) {
  const handleDeactivate = useCallback(
    () => onDeactivate(user),
    [onDeactivate, user],
  );
  const handleReactivate = useCallback(
    () => onReactivate(user),
    [onReactivate, user],
  );
  const handleSendPasswordReset = useCallback(
    () => onSendPasswordReset(user),
    [onSendPasswordReset, user],
  );
  const handleAppointOwner = useCallback(
    () => onAppointOwner(user),
    [onAppointOwner, user],
  );
  const { capabilities } = user;
  const hasActions =
    capabilities.can_deactivate ||
    capabilities.can_reactivate ||
    capabilities.can_send_password_reset ||
    capabilities.can_appoint_owner;

  return (
    <Flex
      as="li"
      direction="row"
      justifyContent="space-between"
      alignItems="center"
      gap="md"
      aria-label={user.email}
    >
      <Flex direction="column">
        <Flex direction="row" alignItems="center" gap="xs">
          <Text fontSize="sm">{user.full_name}</Text>
          {user.status === "DEACTIVATED" && (
            <Badge variant="subtle">Deactivated</Badge>
          )}
        </Flex>
        <Text fontSize="xs" color="subtle">
          {user.email}
        </Text>
        <Text fontSize="xs" color="subtle">
          {signInMethodsLabel(user)}
        </Text>
        <Text fontSize="xs" color="subtle">
          {tenantsLabel(user)}
        </Text>
      </Flex>
      {hasActions && (
        <DropdownMenu modal={false}>
          <DropdownMenu.Trigger>
            <IconButton
              type="more_horiz"
              backgroundColor="default"
              iconColor="black"
              iconSize="md"
              disabled={isUpdating}
              aria-label={`${USER_ACTIONS_LABEL} for ${user.full_name}`}
            />
          </DropdownMenu.Trigger>
          <DropdownMenu.Content>
            {capabilities.can_send_password_reset && (
              <DropdownMenu.Item icon="email" onClick={handleSendPasswordReset}>
                Send password reset
              </DropdownMenu.Item>
            )}
            {capabilities.can_appoint_owner && (
              <DropdownMenu.Item icon="key" onClick={handleAppointOwner}>
                Make owner of a tenant
              </DropdownMenu.Item>
            )}
            {capabilities.can_reactivate && (
              <DropdownMenu.Item icon="refresh" onClick={handleReactivate}>
                Reactivate
              </DropdownMenu.Item>
            )}
            {capabilities.can_deactivate && (
              <DropdownMenu.Item icon="circle_x" onClick={handleDeactivate}>
                Deactivate
              </DropdownMenu.Item>
            )}
          </DropdownMenu.Content>
        </DropdownMenu>
      )}
    </Flex>
  );
}

export function Users() {
  const [searchText, setSearchText] = useState("");
  const [page, setPage] = useState(FIRST_PAGE);
  const search = useDebouncedValue(searchText, SEARCH_DEBOUNCE_MILLISECONDS);
  const users = usePlatformUsers({ search, page });
  const deactivateUser = useDeactivateUser();
  const reactivateUser = useReactivateUser();
  const sendPasswordReset = useSendPasswordReset();
  const [userPendingDeactivation, setUserPendingDeactivation] =
    useState<PlatformUserDto | null>(null);
  const [userPendingAppointment, setUserPendingAppointment] =
    useState<PlatformUserDto | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [tenantsWithoutOwner, setTenantsWithoutOwner] = useState<
    TenantSummaryDto[]
  >([]);

  const { mutate: deactivate, reset: resetDeactivation } = deactivateUser;
  const { mutate: reactivate } = reactivateUser;
  const { mutate: sendReset } = sendPasswordReset;

  const handleSearchChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setSearchText(event.target.value);
      setPage(FIRST_PAGE);
    },
    [],
  );
  const handleDeactivate = useCallback(
    (user: PlatformUserDto) => {
      resetDeactivation();
      setUserPendingDeactivation(user);
    },
    [resetDeactivation],
  );
  const handleCloseDeactivation = useCallback(
    () => setUserPendingDeactivation(null),
    [],
  );
  const handleConfirmDeactivation = useCallback(() => {
    if (!userPendingDeactivation) {
      throw new Error("No user is pending deactivation");
    }
    setNotice(null);
    deactivate(userPendingDeactivation.id, {
      onSuccess: (deactivated) => {
        setUserPendingDeactivation(null);
        setNotice(
          `${userPendingDeactivation.full_name} is deactivated and signed out everywhere.`,
        );
        setTenantsWithoutOwner(deactivated.tenants_without_active_owner);
      },
    });
  }, [deactivate, userPendingDeactivation]);
  const handleReactivate = useCallback(
    (user: PlatformUserDto) => {
      setNotice(null);
      reactivate(user.id, {
        onSuccess: () => setNotice(`${user.full_name} is reactivated.`),
      });
    },
    [reactivate],
  );
  const handleSendPasswordReset = useCallback(
    (user: PlatformUserDto) => {
      setNotice(null);
      sendReset(user.id, {
        onSuccess: () =>
          setNotice(`A password reset email is queued for ${user.email}.`),
      });
    },
    [sendReset],
  );
  const handleCloseAppointment = useCallback(
    () => setUserPendingAppointment(null),
    [],
  );
  const handleAppointed = useCallback(
    (tenant: PlatformTenantDto) => {
      if (!userPendingAppointment) {
        throw new Error("No user is pending appointment");
      }
      setUserPendingAppointment(null);
      setNotice(
        `${userPendingAppointment.full_name} is now an owner of ${tenant.name}.`,
      );
      setTenantsWithoutOwner((tenants) =>
        tenants.filter(
          (tenantWithoutOwner) =>
            tenantWithoutOwner.tenant_id !== tenant.tenant_id,
        ),
      );
    },
    [userPendingAppointment],
  );

  const failure =
    users.error ?? reactivateUser.error ?? sendPasswordReset.error;

  return (
    <Flex direction="column" gap="xl" overflow="scrollY">
      <Heading accessibilityLevel={2} size="2xl" textColor="black">
        Users
      </Heading>
      <Box sx={SEARCH_INPUT_STYLE}>
        <Input
          value={searchText}
          onChange={handleSearchChange}
          placeholder="Search by name or email"
          leadingIcon="search"
          aria-label="Search users"
        />
      </Box>
      {notice && <BannerSlim type="success" message={notice} />}
      {tenantsWithoutOwner.length > 0 && (
        <BannerSlim
          type="warning"
          message={tenantsWithoutOwnerMessage(tenantsWithoutOwner)}
        />
      )}
      {failure && <BannerSlim type="error" message={failure.message} />}
      {users.data &&
        (users.data.users.length === 0 ? (
          <Text fontSize="sm" color="subtle">
            No users match your search.
          </Text>
        ) : (
          <Flex as="ul" direction="column" gap="md" sx={UNSTYLED_LIST_STYLE}>
            {users.data.users.map((user) => (
              <UserRow
                key={user.id}
                user={user}
                onDeactivate={handleDeactivate}
                onReactivate={handleReactivate}
                onSendPasswordReset={handleSendPasswordReset}
                onAppointOwner={setUserPendingAppointment}
                isUpdating={
                  reactivateUser.isPending || sendPasswordReset.isPending
                }
              />
            ))}
          </Flex>
        ))}
      {users.data && (
        <PlatformPagination listPage={users.data} onPageChange={setPage} />
      )}
      {userPendingDeactivation && (
        <DeactivateUserDialog
          user={userPendingDeactivation}
          onConfirm={handleConfirmDeactivation}
          onClose={handleCloseDeactivation}
          isPending={deactivateUser.isPending}
          error={deactivateUser.error}
        />
      )}
      {userPendingAppointment && (
        <AppointOwnerDialog
          user={userPendingAppointment}
          onAppointed={handleAppointed}
          onClose={handleCloseAppointment}
        />
      )}
    </Flex>
  );
}
