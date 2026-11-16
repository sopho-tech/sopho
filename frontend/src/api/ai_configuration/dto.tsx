import { AllowanceUsage } from "src/api/ai_usage";

export type Provider = "anthropic" | "openai";
export type AiConfigurationStatus =
  "not_configured" | "untested" | "live" | "failed";

export type KeySource = "TENANT" | "PLATFORM";

export type PlatformAiAllowance = {
  allowance_usage: AllowanceUsage;
  tokens_left_label: string;
  is_allowance_used: boolean;
};

export type AiConfiguration = {
  status: AiConfigurationStatus;
  provider?: Provider;
  last_checked_at?: string;
  is_ai_available: boolean;
  key_source: KeySource | null;
  unavailable_message: string | null;
  platform_ai: PlatformAiAllowance | null;
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
