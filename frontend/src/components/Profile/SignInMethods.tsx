import { useCallback, useState } from "react";
import { useSearchParams } from "react-router";
import {
  BannerSlim,
  Button,
  Flex,
  Heading,
  Table,
  Text,
} from "src/components/design-system";
import {
  SignInProviderDto,
  connectProviderUrl,
  refreshSessionTokens,
  useDisconnectProvider,
  useMySignInMethods,
} from "src/api/auth/queries";
import { ChangePasswordDialog } from "src/components/Profile/ChangePasswordDialog";
import { DisconnectProviderDialog } from "src/components/Profile/DisconnectProviderDialog";
import {
  CONNECTED_PROVIDER_PARAMETER,
  CONNECTED_RESULT,
  CONNECT_RESULT_MESSAGES,
  CONNECT_RESULT_PARAMETER,
  PASSWORD_CHANGED_MESSAGE,
  SIGN_IN_METHODS_TABLE_COLUMNS,
} from "src/components/Profile/constants";
import { APP_ROUTES } from "src/constants/app_routes";

async function connectProvider(providerName: string) {
  const isSessionFresh = await refreshSessionTokens();
  window.location.assign(
    isSessionFresh ? connectProviderUrl(providerName) : APP_ROUTES.SIGN_IN,
  );
}

function ProviderRow({
  provider,
  onDisconnect,
}: {
  provider: SignInProviderDto;
  onDisconnect: (provider: SignInProviderDto) => void;
}) {
  const handleConnect = useCallback(() => {
    void connectProvider(provider.name);
  }, [provider.name]);
  const handleDisconnect = useCallback(
    () => onDisconnect(provider),
    [onDisconnect, provider],
  );

  return (
    <Table.Row aria-label={provider.display_name}>
      <Table.Cell>
        <Flex direction="column" overflow="hidden">
          <Text fontSize="sm">{provider.display_name}</Text>
          <Text fontSize="xs" color="subtle" truncate>
            {provider.connected
              ? `Connected as ${provider.email ?? provider.display_name}`
              : "Not connected"}
          </Text>
          {provider.disconnect_blocked_reason && (
            <Text fontSize="xs" color="subtle">
              {provider.disconnect_blocked_reason}
            </Text>
          )}
        </Flex>
      </Table.Cell>
      <Table.Cell align="end">
        {provider.connected ? (
          <Button
            label="Disconnect"
            shape="rectangle"
            size="sm"
            backgroundColor="white"
            onClick={handleDisconnect}
            disabled={!provider.can_disconnect}
          />
        ) : (
          <Button
            label="Connect"
            shape="rectangle"
            size="sm"
            backgroundColor="accent"
            onClick={handleConnect}
          />
        )}
      </Table.Cell>
    </Table.Row>
  );
}

export function SignInMethods() {
  const [searchParams] = useSearchParams();
  const connectResult = searchParams.get(CONNECT_RESULT_PARAMETER);
  const connectedProviderName = searchParams.get(CONNECTED_PROVIDER_PARAMETER);
  const mySignInMethods = useMySignInMethods();
  const disconnectProvider = useDisconnectProvider();
  const { mutate: disconnect, reset: resetDisconnect } = disconnectProvider;
  const [providerPendingDisconnect, setProviderPendingDisconnect] =
    useState<SignInProviderDto | null>(null);
  const [disconnectedProviderName, setDisconnectedProviderName] = useState<
    string | null
  >(null);
  const handleDisconnect = useCallback(
    (provider: SignInProviderDto) => {
      resetDisconnect();
      setProviderPendingDisconnect(provider);
    },
    [resetDisconnect],
  );
  const handleCloseDisconnect = useCallback(
    () => setProviderPendingDisconnect(null),
    [],
  );
  const handleConfirmDisconnect = useCallback(() => {
    if (!providerPendingDisconnect) {
      throw new Error("No provider is pending disconnect");
    }
    disconnect(providerPendingDisconnect.name, {
      onSuccess: () => {
        setDisconnectedProviderName(providerPendingDisconnect.display_name);
        setProviderPendingDisconnect(null);
      },
    });
  }, [disconnect, providerPendingDisconnect]);
  const [isChangingPassword, setIsChangingPassword] = useState(false);
  const [isPasswordChanged, setIsPasswordChanged] = useState(false);
  const handleChangePassword = useCallback(() => {
    setIsPasswordChanged(false);
    setIsChangingPassword(true);
  }, []);
  const handleCloseChangePassword = useCallback(
    () => setIsChangingPassword(false),
    [],
  );
  const handlePasswordChanged = useCallback(() => {
    setIsChangingPassword(false);
    setIsPasswordChanged(true);
  }, []);

  const connectedProvider = mySignInMethods.data?.providers.find(
    (provider) => provider.name === connectedProviderName,
  );
  const connectFailureMessage =
    connectResult && connectResult !== CONNECTED_RESULT
      ? CONNECT_RESULT_MESSAGES[connectResult]
      : undefined;

  return (
    <Flex direction="column" gap="sm">
      <Heading accessibilityLevel={3} size="lg">
        Sign-in methods
      </Heading>
      {isPasswordChanged && (
        <BannerSlim type="success" message={PASSWORD_CHANGED_MESSAGE} />
      )}
      {connectResult === CONNECTED_RESULT && connectedProvider && (
        <BannerSlim
          type="success"
          message={`${connectedProvider.display_name} is connected. You can now sign in with it.`}
        />
      )}
      {connectFailureMessage && (
        <BannerSlim type="error" message={connectFailureMessage} />
      )}
      {mySignInMethods.error && (
        <BannerSlim type="error" message={mySignInMethods.error.message} />
      )}
      {disconnectedProviderName && (
        <BannerSlim
          type="success"
          message={`${disconnectedProviderName} is disconnected.`}
        />
      )}
      {mySignInMethods.data && (
        <Table
          columns={SIGN_IN_METHODS_TABLE_COLUMNS}
          aria-label="Sign-in methods"
        >
          {mySignInMethods.data.password_available && (
            <Table.Row aria-label="Password">
              <Table.Cell>
                <Flex direction="column" overflow="hidden">
                  <Text fontSize="sm">Password</Text>
                  <Text fontSize="xs" color="subtle">
                    {mySignInMethods.data.has_password
                      ? "You can sign in with your password"
                      : "Not set. You sign in with single sign-on"}
                  </Text>
                </Flex>
              </Table.Cell>
              <Table.Cell align="end">
                {mySignInMethods.data.has_password && (
                  <Button
                    label="Change password"
                    shape="rectangle"
                    size="sm"
                    backgroundColor="white"
                    onClick={handleChangePassword}
                  />
                )}
              </Table.Cell>
            </Table.Row>
          )}
          {mySignInMethods.data.providers.map((provider) => (
            <ProviderRow
              key={provider.name}
              provider={provider}
              onDisconnect={handleDisconnect}
            />
          ))}
        </Table>
      )}
      {isChangingPassword && (
        <ChangePasswordDialog
          onChanged={handlePasswordChanged}
          onClose={handleCloseChangePassword}
        />
      )}
      {providerPendingDisconnect && (
        <DisconnectProviderDialog
          provider={providerPendingDisconnect}
          onConfirm={handleConfirmDisconnect}
          onClose={handleCloseDisconnect}
          isPending={disconnectProvider.isPending}
          error={disconnectProvider.error}
        />
      )}
    </Flex>
  );
}
