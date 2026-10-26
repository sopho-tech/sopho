import { useCallback, useState } from "react";
import { useSearchParams } from "react-router";
import {
  BannerSlim,
  Button,
  Flex,
  Heading,
  Text,
} from "src/components/design-system";
import {
  SignInProviderDto,
  connectProviderUrl,
  refreshSessionTokens,
  useDisconnectProvider,
  useSignInMethods,
} from "src/api/auth/queries";
import { ChangePasswordForm } from "src/components/Profile/ChangePasswordForm";
import {
  CONNECTED_PROVIDER_PARAMETER,
  CONNECTED_RESULT,
  CONNECT_RESULT_MESSAGES,
  CONNECT_RESULT_PARAMETER,
} from "src/components/Profile/constants";
import { UNSTYLED_LIST_STYLE } from "src/components/Settings/Members/constants";
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
  isDisconnecting,
}: {
  provider: SignInProviderDto;
  onDisconnect: (providerName: string) => void;
  isDisconnecting: boolean;
}) {
  const handleConnect = useCallback(() => {
    void connectProvider(provider.name);
  }, [provider.name]);
  const handleDisconnect = useCallback(
    () => onDisconnect(provider.name),
    [onDisconnect, provider.name],
  );

  return (
    <Flex
      as="li"
      direction="row"
      justifyContent="space-between"
      alignItems="center"
      gap="md"
      aria-label={provider.display_name}
    >
      <Flex direction="column">
        <Text fontSize="sm">{provider.display_name}</Text>
        <Text fontSize="xs" color="subtle">
          {provider.connected
            ? `Connected as ${provider.email ?? provider.display_name}`
            : "Not connected"}
        </Text>
      </Flex>
      {provider.connected ? (
        <Button
          label="Disconnect"
          shape="rectangle"
          size="sm"
          backgroundColor="white"
          onClick={handleDisconnect}
          disabled={!provider.can_disconnect || isDisconnecting}
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
    </Flex>
  );
}

export function SignInMethods() {
  const [searchParams] = useSearchParams();
  const connectResult = searchParams.get(CONNECT_RESULT_PARAMETER);
  const connectedProviderName = searchParams.get(CONNECTED_PROVIDER_PARAMETER);
  const signInMethods = useSignInMethods();
  const disconnectProvider = useDisconnectProvider();
  const { mutate: disconnect } = disconnectProvider;
  const [isChangingPassword, setIsChangingPassword] = useState(false);
  const handleChangePassword = useCallback(
    () => setIsChangingPassword(true),
    [],
  );

  const connectedProvider = signInMethods.data?.providers.find(
    (provider) => provider.name === connectedProviderName,
  );
  const connectFailureMessage =
    connectResult && connectResult !== CONNECTED_RESULT
      ? CONNECT_RESULT_MESSAGES[connectResult]
      : undefined;

  return (
    <Flex direction="column" gap="md">
      <Heading accessibilityLevel={2} size="lg">
        Sign-in methods
      </Heading>
      {connectResult === CONNECTED_RESULT && connectedProvider && (
        <BannerSlim
          type="success"
          message={`${connectedProvider.display_name} is connected. You can now sign in with it.`}
        />
      )}
      {connectFailureMessage && (
        <BannerSlim type="error" message={connectFailureMessage} />
      )}
      {signInMethods.error && (
        <BannerSlim type="error" message={signInMethods.error.message} />
      )}
      {disconnectProvider.error && (
        <BannerSlim type="error" message={disconnectProvider.error.message} />
      )}
      {signInMethods.data && (
        <Flex as="ul" direction="column" gap="sm" sx={UNSTYLED_LIST_STYLE}>
          <Flex
            as="li"
            direction="row"
            justifyContent="space-between"
            alignItems="center"
            gap="md"
            aria-label="Password"
          >
            <Flex direction="column">
              <Text fontSize="sm">Password</Text>
              <Text fontSize="xs" color="subtle">
                {signInMethods.data.has_password
                  ? "You can sign in with your password"
                  : "Not set. You sign in with single sign-on"}
              </Text>
            </Flex>
            {signInMethods.data.has_password && !isChangingPassword && (
              <Button
                label="Change password"
                shape="rectangle"
                size="sm"
                backgroundColor="white"
                onClick={handleChangePassword}
              />
            )}
          </Flex>
          {isChangingPassword && <ChangePasswordForm />}
          {signInMethods.data.providers.map((provider) => (
            <ProviderRow
              key={provider.name}
              provider={provider}
              onDisconnect={disconnect}
              isDisconnecting={disconnectProvider.isPending}
            />
          ))}
        </Flex>
      )}
    </Flex>
  );
}
