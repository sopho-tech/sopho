import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Badge,
  BannerSlim,
  Button,
  DropdownMenu,
  Flex,
  Heading,
  IconButton,
  Table,
  Text,
  type SelectOption,
} from "src/components/design-system";
import {
  Form,
  useFormCompoundContext,
} from "src/components/design-system/Form";
import {
  AiConfiguration,
  AiConfigurationStatus,
  Provider,
  useAiConfiguration,
  useDeleteAiConfiguration,
  useTestAiConfiguration,
  useUpdateAiConfiguration,
} from "src/api/ai_configuration";
import styles from "src/components/Settings/AIConfiguration/AIConfiguration.module.css";
import { RemoveAiConfigurationDialog } from "src/components/Settings/AIConfiguration/RemoveAiConfigurationDialog";
import { formatTimestamp } from "src/utils/timestamp_utils";

type ProviderOption = { value: Provider; label: string };

const PROVIDER_OPTIONS: ProviderOption[] = [
  { value: "anthropic", label: "Anthropic" },
  { value: "openai", label: "OpenAI" },
];

const PROVIDER_LABELS = Object.fromEntries(
  PROVIDER_OPTIONS.map(({ value, label }) => [value, label]),
) as Record<Provider, string>;

const PROVIDER_SELECT_OPTIONS: SelectOption[] = PROVIDER_OPTIONS.map((o) => ({
  value: o.value,
  label: o.label,
}));

type AiConfigFormValues = {
  provider: Provider;
  api_key: string;
};

const CONNECTION_STATUS_BADGES: Record<
  Exclude<AiConfigurationStatus, "not_configured">,
  { label: string; variant: "green" | "red" | "subtle" }
> = {
  live: { label: "Verified", variant: "green" },
  failed: { label: "Failed", variant: "red" },
  untested: { label: "Not tested", variant: "subtle" },
};

const PROVIDER_TABLE_COLUMNS = "minmax(0, 1fr) 6rem 11rem 2.5rem";

function CurrentProviderTable({
  configuration,
  onRemove,
}: {
  configuration: AiConfiguration;
  onRemove: () => void;
}) {
  if (configuration.status === "not_configured") {
    return (
      <BannerSlim
        type="warning"
        message="No provider is set up yet. Add one below to turn on AI features."
      />
    );
  }
  if (!configuration.provider) {
    throw new Error("A configured AI provider has no provider name");
  }
  const statusBadge = CONNECTION_STATUS_BADGES[configuration.status];
  return (
    <Table columns={PROVIDER_TABLE_COLUMNS} aria-label="Current provider">
      <Table.Header>
        <Table.HeaderCell>Provider</Table.HeaderCell>
        <Table.HeaderCell>Status</Table.HeaderCell>
        <Table.HeaderCell>Last checked</Table.HeaderCell>
        <Table.HeaderCell />
      </Table.Header>
      <Table.Row aria-label={PROVIDER_LABELS[configuration.provider]}>
        <Table.Cell>
          <Text fontSize="sm">{PROVIDER_LABELS[configuration.provider]}</Text>
        </Table.Cell>
        <Table.Cell>
          <Badge variant={statusBadge.variant}>{statusBadge.label}</Badge>
        </Table.Cell>
        <Table.Cell>
          <Text fontSize="sm" color="darkGrey">
            {configuration.last_checked_at
              ? formatTimestamp(configuration.last_checked_at)
              : "Never"}
          </Text>
        </Table.Cell>
        <Table.Cell align="end">
          <DropdownMenu modal={false}>
            <DropdownMenu.Trigger>
              <IconButton
                type="more_horiz"
                backgroundColor="default"
                iconColor="grey"
                iconSize="md"
                aria-label="Provider actions"
              />
            </DropdownMenu.Trigger>
            <DropdownMenu.Content>
              <DropdownMenu.Item icon="delete" onClick={onRemove}>
                Remove provider
              </DropdownMenu.Item>
            </DropdownMenu.Content>
          </DropdownMenu>
        </Table.Cell>
      </Table.Row>
    </Table>
  );
}

function ClearApiKeyAfterSuccess({ tick }: { tick: number }) {
  const { form } = useFormCompoundContext();

  useEffect(() => {
    if (tick <= 0) return;
    const v = form.state.values as AiConfigFormValues;
    form.reset({ ...v, api_key: "" });
  }, [tick, form]);

  return null;
}

function AiConfigurationActions({
  testMutation,
  updateMutation,
}: {
  testMutation: ReturnType<typeof useTestAiConfiguration>;
  updateMutation: ReturnType<typeof useUpdateAiConfiguration>;
}) {
  const { form } = useFormCompoundContext();

  return (
    <form.Subscribe
      selector={(s) => {
        const values = s.values as AiConfigFormValues;
        return {
          provider: values.provider,
          api_key: String(values.api_key ?? ""),
          isSubmitting: s.isSubmitting,
        };
      }}
    >
      {({ provider, api_key, isSubmitting }) => {
        const canAct = api_key.trim().length > 0;
        const handleTest = () => {
          if (!canAct || testMutation.isPending) return;
          testMutation.mutate({ provider, api_key });
        };
        return (
          <>
            <Button
              label="Test connection"
              backgroundColor="white"
              shape="rectangle"
              size="sm"
              onClick={handleTest}
              disabled={!canAct || testMutation.isPending}
            />
            <Button
              label="Save"
              backgroundColor="accent"
              shape="rectangle"
              size="sm"
              type="submit"
              onClick={() => {
                void form.handleSubmit();
              }}
              disabled={!canAct || updateMutation.isPending || isSubmitting}
            />
          </>
        );
      }}
    </form.Subscribe>
  );
}

export function AIConfiguration() {
  const { data: configuration } = useAiConfiguration();
  const updateMutation = useUpdateAiConfiguration();
  const deleteMutation = useDeleteAiConfiguration();
  const testMutation = useTestAiConfiguration();
  const [credentialsTick, setCredentialsTick] = useState(0);
  const [isRemoving, setIsRemoving] = useState(false);
  const isConfigured = configuration?.status !== "not_configured";
  const testResult = testMutation.data;
  const apiKeyPlaceholder = isConfigured
    ? "Enter a new key to replace the stored one"
    : "Provider API key";

  const defaultFormValues = useMemo<AiConfigFormValues>(
    () => ({
      provider:
        configuration?.status !== "not_configured"
          ? (configuration?.provider ?? "anthropic")
          : "anthropic",
      api_key: "",
    }),
    [configuration?.status, configuration?.provider],
  );

  const handleSubmit = useCallback(
    (values: Record<string, unknown>) => {
      const api_key = values.api_key;
      const provider = values.provider;
      if (typeof api_key !== "string" || typeof provider !== "string") return;
      updateMutation.mutate(
        { provider: provider as Provider, api_key },
        {
          onSuccess: () => {
            setCredentialsTick((t) => t + 1);
          },
        },
      );
    },
    [updateMutation],
  );

  const handleOpenRemove = useCallback(() => {
    deleteMutation.reset();
    setIsRemoving(true);
  }, [deleteMutation]);
  const handleCloseRemove = useCallback(() => setIsRemoving(false), []);
  const handleConfirmRemove = useCallback(() => {
    deleteMutation.mutate(undefined, {
      onSuccess: () => {
        setIsRemoving(false);
        setCredentialsTick((tick) => tick + 1);
      },
    });
  }, [deleteMutation]);

  return (
    <Flex direction="column" gap="lg" width="100%" className={styles.page}>
      <Flex direction="column" gap="2xs">
        <Heading accessibilityLevel={2} size="2xl" textColor="black">
          AI Configurations
        </Heading>
        <Text fontSize="sm" color="subtle">
          Configure the LLM provider used for AI features.
        </Text>
      </Flex>

      {configuration && (
        <CurrentProviderTable
          configuration={configuration}
          onRemove={handleOpenRemove}
        />
      )}

      <Form defaultValues={defaultFormValues} onSubmit={handleSubmit}>
        <ClearApiKeyAfterSuccess tick={credentialsTick} />
        <Form.ErrorBanner />
        <Form.Fields className={styles.fields}>
          <Form.Field name="provider" className={styles.field}>
            <Form.Label>Provider</Form.Label>
            <Form.Select
              placeholder="Provider"
              options={PROVIDER_SELECT_OPTIONS}
            />
          </Form.Field>
          <Form.Field
            name="api_key"
            required
            errorMessage="API key is required"
            className={styles.field}
          >
            <Form.Label>API key</Form.Label>
            <Form.Password
              placeholder={apiKeyPlaceholder}
              className={styles.fullWidthPasswordShell}
              inputContainerClassName={styles.fullWidthPasswordContainer}
            />
          </Form.Field>
        </Form.Fields>

        <div className={styles.feedback}>
          {testMutation.isPending && (
            <Text fontSize="sm" color="subtle">
              Testing…
            </Text>
          )}
          {testResult && (
            <BannerSlim
              type={testResult.ok ? "success" : "warning"}
              message={
                testResult.ok
                  ? "Credentials are working"
                  : `Failed: ${testResult.error ?? "unknown error"}`
              }
            />
          )}
        </div>

        <Form.Actions className={styles.actions}>
          <AiConfigurationActions
            testMutation={testMutation}
            updateMutation={updateMutation}
          />
        </Form.Actions>
      </Form>
      {isRemoving && configuration?.provider && (
        <RemoveAiConfigurationDialog
          providerLabel={PROVIDER_LABELS[configuration.provider]}
          onConfirm={handleConfirmRemove}
          onClose={handleCloseRemove}
          isPending={deleteMutation.isPending}
          error={deleteMutation.error}
        />
      )}
    </Flex>
  );
}

AIConfiguration.displayName = "AIConfiguration";
