import { ChangeEvent, useCallback, useState } from "react";
import {
  Avatar,
  Badge,
  BannerSlim,
  Box,
  DropdownMenu,
  Flex,
  Heading,
  IconButton,
  Input,
  Table,
  Text,
} from "src/components/design-system";
import {
  PlatformTenantDto,
  PlatformUserDto,
  TenantSummaryDto,
  useDeactivateUser,
  useMakeAdministrator,
  useRemoveAdministrator,
  usePlatformUsers,
  useReactivateUser,
  useSendPasswordReset,
} from "src/api/platform/queries";
import { AdministratorRoleDialog } from "src/components/Platform/AdministratorRoleDialog";
import { AppointOwnerDialog } from "src/components/Platform/AppointOwnerDialog";
import { DeactivateUserDialog } from "src/components/Platform/DeactivateUserDialog";
import { PlatformPagination } from "src/components/Platform/PlatformPagination";
import { UserTenants } from "src/components/Platform/UserTenants";
import {
  FIRST_PAGE,
  SEARCH_DEBOUNCE_MILLISECONDS,
  SEARCH_INPUT_STYLE,
  USER_ACTIONS_LABEL,
  USERS_TABLE_COLUMNS,
} from "src/components/Platform/constants";
import { useDebouncedValue } from "src/utils/hooks";

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
  onChangeAdministratorRole: (user: PlatformUserDto) => void;
  isUpdating: boolean;
};

function UserRow({
  user,
  onDeactivate,
  onReactivate,
  onSendPasswordReset,
  onAppointOwner,
  onChangeAdministratorRole,
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
  const handleChangeAdministratorRole = useCallback(
    () => onChangeAdministratorRole(user),
    [onChangeAdministratorRole, user],
  );
  const { capabilities } = user;
  const hasActions =
    capabilities.can_make_administrator ||
    capabilities.can_remove_administrator ||
    capabilities.can_deactivate ||
    capabilities.can_reactivate ||
    capabilities.can_send_password_reset ||
    capabilities.can_appoint_owner;

  return (
    <Table.Row aria-label={user.email}>
      <Table.Cell>
        <Avatar name={user.full_name} />
        <Flex direction="column" overflow="hidden">
          <Flex direction="row" alignItems="center" gap="xs">
            <Text fontSize="sm" truncate>
              {user.full_name}
            </Text>
            {user.is_current_user && <Badge variant="subtle">You</Badge>}
            {user.is_platform_administrator && (
              <Badge variant="command">Administrator</Badge>
            )}
            {user.status === "DEACTIVATED" && (
              <Badge variant="yellow">Deactivated</Badge>
            )}
          </Flex>
          <Text fontSize="xs" color="subtle" truncate>
            {user.email}
          </Text>
        </Flex>
      </Table.Cell>
      <Table.Cell>
        <Flex direction="row" gap="2xs" flexWrap="wrap">
          {user.has_password && <Badge variant="subtle">Password</Badge>}
          {user.provider_names.map((providerName) => (
            <Badge key={providerName} variant="subtle">
              {providerName}
            </Badge>
          ))}
          {!user.has_password && user.provider_names.length === 0 && (
            <Text fontSize="sm" color="subtle">
              None
            </Text>
          )}
        </Flex>
      </Table.Cell>
      <Table.Cell>
        <UserTenants tenants={user.tenants} />
      </Table.Cell>
      <Table.Cell align="end">
        {hasActions && (
          <DropdownMenu modal={false}>
            <DropdownMenu.Trigger>
              <IconButton
                type="more_horiz"
                backgroundColor="default"
                iconColor="grey"
                iconSize="md"
                disabled={isUpdating}
                aria-label={`${USER_ACTIONS_LABEL} for ${user.full_name}`}
              />
            </DropdownMenu.Trigger>
            <DropdownMenu.Content>
              {capabilities.can_send_password_reset && (
                <DropdownMenu.Item
                  icon="email"
                  onClick={handleSendPasswordReset}
                >
                  Send password reset
                </DropdownMenu.Item>
              )}
              {capabilities.can_appoint_owner && (
                <DropdownMenu.Item icon="key" onClick={handleAppointOwner}>
                  Make owner of a tenant
                </DropdownMenu.Item>
              )}
              {capabilities.can_make_administrator && (
                <DropdownMenu.Item
                  icon="user"
                  onClick={handleChangeAdministratorRole}
                >
                  Make administrator
                </DropdownMenu.Item>
              )}
              {capabilities.can_remove_administrator && (
                <DropdownMenu.Item
                  icon="user"
                  onClick={handleChangeAdministratorRole}
                >
                  Remove administrator
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
      </Table.Cell>
    </Table.Row>
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
  const [userPendingAdministratorChange, setUserPendingAdministratorChange] =
    useState<PlatformUserDto | null>(null);
  const makeAdministrator = useMakeAdministrator();
  const removeAdministrator = useRemoveAdministrator();
  const { mutate: makeAdministratorRole, reset: resetMakeAdministrator } =
    makeAdministrator;
  const { mutate: removeAdministratorRole, reset: resetRemoveAdministrator } =
    removeAdministrator;
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
  const handleChangeAdministratorRole = useCallback(
    (user: PlatformUserDto) => {
      resetMakeAdministrator();
      resetRemoveAdministrator();
      setUserPendingAdministratorChange(user);
    },
    [resetMakeAdministrator, resetRemoveAdministrator],
  );
  const handleCloseAdministratorChange = useCallback(
    () => setUserPendingAdministratorChange(null),
    [],
  );
  const handleConfirmAdministratorChange = useCallback(() => {
    if (!userPendingAdministratorChange) {
      throw new Error("No user is pending an administrator change");
    }
    setNotice(null);
    const { id, full_name, is_platform_administrator } =
      userPendingAdministratorChange;
    if (is_platform_administrator) {
      removeAdministratorRole(id, {
        onSuccess: () => {
          setUserPendingAdministratorChange(null);
          setNotice(`${full_name} is no longer an administrator.`);
        },
      });
      return;
    }
    makeAdministratorRole(id, {
      onSuccess: () => {
        setUserPendingAdministratorChange(null);
        setNotice(`${full_name} is now an administrator.`);
      },
    });
  }, [
    makeAdministratorRole,
    removeAdministratorRole,
    userPendingAdministratorChange,
  ]);
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
    <Flex direction="column" gap="lg">
      <Flex direction="column" gap="2xs">
        <Heading accessibilityLevel={2} size="2xl" textColor="black">
          Global Users
        </Heading>
        <Text fontSize="sm" color="subtle">
          Manage every account on this Sopho installation.
        </Text>
      </Flex>
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
          <Table columns={USERS_TABLE_COLUMNS} aria-label="Users">
            <Table.Header>
              <Table.HeaderCell>User</Table.HeaderCell>
              <Table.HeaderCell>Sign-in methods</Table.HeaderCell>
              <Table.HeaderCell>Tenants</Table.HeaderCell>
              <Table.HeaderCell />
            </Table.Header>
            {users.data.users.map((user) => (
              <UserRow
                key={user.id}
                user={user}
                onDeactivate={handleDeactivate}
                onReactivate={handleReactivate}
                onSendPasswordReset={handleSendPasswordReset}
                onAppointOwner={setUserPendingAppointment}
                onChangeAdministratorRole={handleChangeAdministratorRole}
                isUpdating={
                  reactivateUser.isPending || sendPasswordReset.isPending
                }
              />
            ))}
          </Table>
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
      {userPendingAdministratorChange && (
        <AdministratorRoleDialog
          user={userPendingAdministratorChange}
          onConfirm={handleConfirmAdministratorChange}
          onClose={handleCloseAdministratorChange}
          isPending={
            makeAdministrator.isPending || removeAdministrator.isPending
          }
          error={makeAdministrator.error ?? removeAdministrator.error}
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
