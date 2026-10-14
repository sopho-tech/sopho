import {
  SAMPLE_POSTGRES_DATABASE,
  SAMPLE_POSTGRES_HOST_VARIABLE,
  SAMPLE_POSTGRES_PASSWORD,
  SAMPLE_POSTGRES_PORT_VARIABLE,
  SAMPLE_POSTGRES_SCHEMA,
  SAMPLE_POSTGRES_USERNAME,
} from "e2e/support/constants";

export type SamplePostgresDatabase = {
  host: string;
  port: string;
  database: string;
  username: string;
  password: string;
  schema: string;
};

function requiredEnvironmentVariable(name: string): string {
  const value = process.env[name];
  if (!value) {
    throw new Error(`${name} is not set; the global setup starts Postgres`);
  }
  return value;
}

export function samplePostgresDatabase(): SamplePostgresDatabase {
  return {
    host: requiredEnvironmentVariable(SAMPLE_POSTGRES_HOST_VARIABLE),
    port: requiredEnvironmentVariable(SAMPLE_POSTGRES_PORT_VARIABLE),
    database: SAMPLE_POSTGRES_DATABASE,
    username: SAMPLE_POSTGRES_USERNAME,
    password: SAMPLE_POSTGRES_PASSWORD,
    schema: SAMPLE_POSTGRES_SCHEMA,
  };
}
