import { PostgreSqlContainer } from "@testcontainers/postgresql";
import {
  POSTGRES_IMAGE,
  POSTGRES_INIT_SCRIPT_PATH,
  SAMPLE_DATA_PATH,
  SAMPLE_POSTGRES_DATABASE,
  SAMPLE_POSTGRES_HOST_VARIABLE,
  SAMPLE_POSTGRES_PASSWORD,
  SAMPLE_POSTGRES_PORT_VARIABLE,
  SAMPLE_POSTGRES_USERNAME,
} from "e2e/support/constants";

export default async function globalSetup() {
  const samplePostgres = await new PostgreSqlContainer(POSTGRES_IMAGE)
    .withDatabase(SAMPLE_POSTGRES_DATABASE)
    .withUsername(SAMPLE_POSTGRES_USERNAME)
    .withPassword(SAMPLE_POSTGRES_PASSWORD)
    .withCopyFilesToContainer([
      { source: SAMPLE_DATA_PATH, target: POSTGRES_INIT_SCRIPT_PATH },
    ])
    .start();
  process.env[SAMPLE_POSTGRES_HOST_VARIABLE] = samplePostgres.getHost();
  process.env[SAMPLE_POSTGRES_PORT_VARIABLE] = String(samplePostgres.getPort());
  return async () => {
    await samplePostgres.stop();
  };
}
