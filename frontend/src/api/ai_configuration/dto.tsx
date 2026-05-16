export type Provider = "anthropic" | "openai";
export type AiConfigurationStatus =
  | "not_configured"
  | "untested"
  | "live"
  | "failed";

export type AiConfiguration = {
  status: AiConfigurationStatus;
  provider?: Provider;
  last_checked_at?: string;
};

export type UpsertAiConfigurationInput = {
  provider: Provider;
  api_key: string;
};

export type TestAiConfigurationInput = UpsertAiConfigurationInput;

export type TestAiConfigurationResult = {
  ok: boolean;
  error?: string;
};
