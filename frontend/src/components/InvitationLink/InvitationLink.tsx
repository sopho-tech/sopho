import { useCallback, useEffect } from "react";
import { useLocation, useNavigate } from "react-router";
import {
  BannerSlim,
  Button,
  Flex,
  Form,
  Heading,
  Text,
} from "src/components/design-system";
import { AuthPageLayout } from "src/components/AuthPageLayout";
import { authPageStyles } from "src/components/AuthPageLayout/auth_page_styles";
import { SingleSignOnButton } from "src/components/SignIn/SingleSignOnButton";
import {
  INCOMPLETE_INVITATION_LINK_MESSAGE,
  INVITATION_SIGN_UP_FORM_DEFAULT_VALUES,
} from "src/components/InvitationLink/constants";
import { useDeleteSession, useSignInPageOptions } from "src/api/auth/queries";
import { AcceptedInvitationDto } from "src/api/invitation/queries";
import {
  InvitationLinkDto,
  useAcceptInvitationLink,
  useInvitationLink,
  useSignUpFromInvitation,
} from "src/api/invitation_link/queries";
import { APP_ROUTES } from "src/constants/app_routes";
import {
  forgetPendingInvitation,
  rememberPendingInvitation,
} from "src/components/InvitationLink/pending_invitation";
import { linkTokenFromHash } from "src/utils/link_token";
import { tenantPath } from "src/utils/tenant";

type InvitationStepProps = {
  token: string;
  invitation: InvitationLinkDto;
};

function useOpenJoinedTenant() {
  const navigate = useNavigate();
  return useCallback(
    (accepted: AcceptedInvitationDto) => {
      forgetPendingInvitation();
      void navigate(tenantPath(accepted.tenant_id, APP_ROUTES.INDEX), {
        replace: true,
      });
    },
    [navigate],
  );
}

function InvitationSummary({ invitation }: { invitation: InvitationLinkDto }) {
  return (
    <Flex direction="column" gap="xs">
      <Heading accessibilityLevel={1} size="xl" textAlign="center">
        {`Join ${invitation.tenant_name}`}
      </Heading>
      <Text fontSize="sm" color="subtle">
        {`${invitation.invited_by_name} invited ${invitation.email} to join ${invitation.tenant_name} as ${invitation.role_name}.`}
      </Text>
    </Flex>
  );
}

function InvitationSignUp({ token, invitation }: InvitationStepProps) {
  const signInPageOptions = useSignInPageOptions();
  const openJoinedTenant = useOpenJoinedTenant();
  const signUp = useSignUpFromInvitation({ onSuccess: openJoinedTenant });
  const { mutate: createAccount } = signUp;
  const rememberInvitation = useCallback(
    () => rememberPendingInvitation(token),
    [token],
  );
  const handleSubmit = useCallback(
    (values: Record<string, unknown>) => {
      const fullName = values.full_name;
      const password = values.password;
      if (typeof fullName !== "string" || typeof password !== "string") {
        throw new Error("The sign-up form returned values that are not text");
      }
      createAccount({ token, full_name: fullName, password });
    },
    [createAccount, token],
  );
  const providers = signInPageOptions.data?.providers ?? [];

  return (
    <Flex direction="column" gap="sm">
      <InvitationSummary invitation={invitation} />
      <Heading accessibilityLevel={2} size="sm" textColor="subtle">
        Create your account
      </Heading>
      <Form
        defaultValues={INVITATION_SIGN_UP_FORM_DEFAULT_VALUES}
        onSubmit={handleSubmit}
        className={authPageStyles.formRootContainer}
        submitOnEnter={true}
      >
        <Form.Fields className={authPageStyles.fieldsContainer}>
          <Form.Field
            name="full_name"
            required
            errorMessage="Full name is required"
            className={authPageStyles.fieldContainer}
          >
            <Form.Input placeholder="Full name" icon="user" />
          </Form.Field>
          <Form.Field
            name="password"
            required
            errorMessage="Password is required"
            className={authPageStyles.fieldContainer}
          >
            <Form.Password placeholder="Choose a password" icon="lock" />
          </Form.Field>
        </Form.Fields>
        {signUp.error && (
          <BannerSlim type="error" message={signUp.error.message} />
        )}
        <Form.Actions className={authPageStyles.formButtonRowContainer}>
          <Form.Submit label="Create account" />
        </Form.Actions>
      </Form>
      {providers.length > 0 && (
        <Flex direction="column" gap="sm">
          <Flex justifyContent="center">
            <Text fontSize="xs" color="subtle">
              or
            </Text>
          </Flex>
          {providers.map((provider) => (
            <SingleSignOnButton
              key={provider.name}
              provider={provider}
              onBeforeRedirect={rememberInvitation}
            />
          ))}
        </Flex>
      )}
    </Flex>
  );
}

function InvitationSignIn({ token, invitation }: InvitationStepProps) {
  const navigate = useNavigate();
  const handleSignIn = useCallback(() => {
    rememberPendingInvitation(token);
    void navigate(APP_ROUTES.SIGN_IN);
  }, [navigate, token]);

  return (
    <Flex direction="column" gap="md">
      <InvitationSummary invitation={invitation} />
      <Text fontSize="sm">{`${invitation.email} already has a Sopho account. Sign in to join.`}</Text>
      <Button
        label="Sign in to join"
        shape="rectangle"
        size="md"
        backgroundColor="accent"
        fullWidth
        onClick={handleSignIn}
      />
    </Flex>
  );
}

function InvitationJoin({ token, invitation }: InvitationStepProps) {
  const openJoinedTenant = useOpenJoinedTenant();
  const acceptInvitation = useAcceptInvitationLink({
    onSuccess: openJoinedTenant,
  });
  const { mutate: accept } = acceptInvitation;
  const handleJoin = useCallback(() => accept(token), [accept, token]);

  return (
    <Flex direction="column" gap="md">
      <InvitationSummary invitation={invitation} />
      {acceptInvitation.error && (
        <BannerSlim type="error" message={acceptInvitation.error.message} />
      )}
      <Button
        label={`Join ${invitation.tenant_name} as ${invitation.role_name}`}
        shape="rectangle"
        size="md"
        backgroundColor="accent"
        fullWidth
        onClick={handleJoin}
        disabled={acceptInvitation.isPending}
      />
    </Flex>
  );
}

function InvitationForAnotherAccount({
  token,
  invitation,
}: InvitationStepProps) {
  const deleteSession = useDeleteSession();
  const { mutate: signOut } = deleteSession;
  const handleSignOut = useCallback(() => {
    rememberPendingInvitation(token);
    signOut();
  }, [signOut, token]);

  return (
    <Flex direction="column" gap="md">
      <InvitationSummary invitation={invitation} />
      <BannerSlim
        type="info"
        message={`This invitation is for ${invitation.email}. Sign out, then sign in as ${invitation.email} to join.`}
      />
      <Button
        label="Sign out"
        shape="rectangle"
        size="md"
        backgroundColor="accent"
        leadingIconName="logout"
        fullWidth
        onClick={handleSignOut}
        disabled={deleteSession.isPending}
      />
    </Flex>
  );
}

function InvitationLinkSteps({ token }: { token: string }) {
  const invitationLink = useInvitationLink(token);

  if (invitationLink.isPending) return null;
  if (invitationLink.isError) {
    return <BannerSlim type="error" message={invitationLink.error.message} />;
  }
  const invitation = invitationLink.data;
  switch (invitation.next_step) {
    case "SIGN_UP":
      return <InvitationSignUp token={token} invitation={invitation} />;
    case "SIGN_IN":
      return <InvitationSignIn token={token} invitation={invitation} />;
    case "JOIN":
      return <InvitationJoin token={token} invitation={invitation} />;
    case "WRONG_ACCOUNT":
      return (
        <InvitationForAnotherAccount token={token} invitation={invitation} />
      );
  }
}

export function InvitationLink() {
  const location = useLocation();
  const token = linkTokenFromHash(location.hash);

  useEffect(() => {
    forgetPendingInvitation();
  }, []);

  return (
    <AuthPageLayout>
      {token ? (
        <InvitationLinkSteps token={token} />
      ) : (
        <BannerSlim type="error" message={INCOMPLETE_INVITATION_LINK_MESSAGE} />
      )}
    </AuthPageLayout>
  );
}
