import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Badge,
  BannerSlim,
  Button,
  Flex,
  Heading,
  Text,
  type SelectOption,
} from "src/components/design-system";
import {
  Form,
  useFormCompoundContext,
} from "src/components/design-system/Form";
import {
  Provider,
  useAiConfiguration,
  useDeleteAiConfiguration,
  useTestAiConfiguration,
  useUpdateAiConfiguration,
} from "src/api/ai_configuration";
import styles from "src/components/Settings/AIConfiguration/AIConfiguration.module.css";
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

type LivenessLabel = {
  label: string;
  variant: "green" | "yellow" | "subtle";
};

function connectionStatusBadge(status: string | undefined): LivenessLabel {
  console.log("test: ", status);
  switch (status) {
    case "live":
      return { label: "live (connection verified)", variant: "green" };
    case "failed":
      return { label: "connection failed when tested", variant: "yellow" };
    case "untested":
      return { label: "not yet tested", variant: "subtle" };
    default:
      return { label: "Untested", variant: "subtle" };
  }
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
  isConfigured,
  testMutation,
  updateMutation,
  deleteMutation,
  onDeleteSuccess,
}: {
  isConfigured: boolean;
  testMutation: ReturnType<typeof useTestAiConfiguration>;
  updateMutation: ReturnType<typeof useUpdateAiConfiguration>;
  deleteMutation: ReturnType<typeof useDeleteAiConfiguration>;
  onDeleteSuccess: () => void;
}) {
  const { form } = useFormCompoundContext();

  const handleDelete = () => {
    if (!isConfigured || deleteMutation.isPending) return;
    deleteMutation.mutate(undefined, {
      onSuccess: () => {
        onDeleteSuccess();
      },
    });
  };

  return (
    <>
      <Button
        label="Delete"
        backgroundColor="red"
        shape="rectangle"
        size="md"
        onClick={handleDelete}
        disabled={!isConfigured || deleteMutation.isPending}
      />
      <div className={styles.actionsTrailing}>
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
                  label="Test"
                  backgroundColor="lightgrey"
                  shape="rectangle"
                  size="md"
                  onClick={handleTest}
                  disabled={!canAct || testMutation.isPending}
                />
                <Button
                  label="Save"
                  backgroundColor="accent"
                  shape="rectangle"
                  size="md"
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
      </div>
    </>
  );
}

export function AIConfiguration() {
  const { data: configuration } = useAiConfiguration();
  const updateMutation = useUpdateAiConfiguration();
  const deleteMutation = useDeleteAiConfiguration();
  const testMutation = useTestAiConfiguration();
  const [credentialsTick, setCredentialsTick] = useState(0);
  const isConfigured = configuration?.status !== "not_configured";
  const statusBadge = connectionStatusBadge(configuration?.status);
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

  return (
    <Flex direction="column" gap="lg" width="100%" className={styles.page}>
      <Flex direction="column" gap="2xs">
        <Heading accessbilityLevel={2} size="2xl" textColor="black">
          AI Configurations
        </Heading>
        <Text fontSize="sm" color="subtle">
          Configure the LLM provider used for AI features.
        </Text>
      </Flex>

      <Flex alignItems="center" gap="sm">
        {isConfigured ? (
          <>
            <Badge variant={statusBadge.variant}>
              {`${PROVIDER_LABELS[configuration?.provider ?? "anthropic"]} — ${statusBadge.label}`}
            </Badge>
            {configuration?.last_checked_at && (
              <Text fontSize="xs" color="subtle">
                {`Verified at ${formatTimestamp(configuration.last_checked_at) ?? ""}`}
              </Text>
            )}
          </>
        ) : (
          <Text fontSize="sm" color="subtle">
            No LLM providers are configured
          </Text>
        )}
      </Flex>

      <Form defaultValues={defaultFormValues} onSubmit={handleSubmit}>
        <ClearApiKeyAfterSuccess tick={credentialsTick} />
        <Form.ErrorBanner />
        <Form.Fields className={styles.fields}>
          <Form.Field name="provider" className={styles.field}>
            <Form.Label>Provider</Form.Label>
            <Form.Select
              groupName="Provider"
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
            isConfigured={isConfigured}
            testMutation={testMutation}
            updateMutation={updateMutation}
            deleteMutation={deleteMutation}
            onDeleteSuccess={() => setCredentialsTick((t) => t + 1)}
          />
        </Form.Actions>
      </Form>
    </Flex>
  );
}

AIConfiguration.displayName = "AIConfiguration";
