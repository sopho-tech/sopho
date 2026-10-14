import {
  PostgreSqlContainer,
  type StartedPostgreSqlContainer,
} from "@testcontainers/postgresql";
import {
  GenericContainer,
  Network,
  Wait,
  getContainerRuntimeClient,
  type StartedNetwork,
  type StartedTestContainer,
} from "testcontainers";
import {
  ADMIN_EMAIL,
  ADMIN_FULL_NAME,
  ADMIN_PASSWORD,
  ADMIN_USERNAME,
  ENCRYPTION_KEY,
  HEALTH_PATH,
  METADATA_POSTGRES_ALIAS,
  METADATA_POSTGRES_DATABASE,
  METADATA_POSTGRES_PASSWORD,
  METADATA_POSTGRES_USERNAME,
  POSTGRES_IMAGE,
  SOPHO_CONTAINER_DATA_DIRECTORY,
  SOPHO_CONTAINER_PORT,
  SOPHO_CONTAINER_STARTUP_TIMEOUT_MILLISECONDS,
} from "e2e/support/constants";

export type StartedSopho = {
  baseUrl: string;
  logs: string[];
  container: StartedTestContainer;
};

export type SophoContainerOptions = {
  image: string;
  environment: Record<string, string>;
  network: StartedNetwork | null;
  dataVolumeName: string | null;
};

export type MetadataPostgres = {
  network: StartedNetwork;
  postgres: StartedPostgreSqlContainer;
};

export function requiredEnvironmentVariable(name: string): string {
  const value = process.env[name];
  if (!value) {
    throw new Error(`${name} must be set to run the image projects`);
  }
  return value;
}

export function readmeEnvironment(): Record<string, string> {
  return {
    ADMIN_USERNAME,
    ADMIN_PASSWORD,
    ADMIN_EMAIL,
    ADMIN_FULL_NAME,
  };
}

export function upgradeEnvironment(
  databaseUrl: string | null,
): Record<string, string> {
  const environment = { ...readmeEnvironment(), ENCRYPTION_KEY };
  return databaseUrl === null
    ? environment
    : { ...environment, DATABASE_URL: databaseUrl };
}

export async function startSopho({
  image,
  environment,
  network,
  dataVolumeName,
}: SophoContainerOptions): Promise<StartedSopho> {
  const logs: string[] = [];
  let container = new GenericContainer(image)
    .withEnvironment(environment)
    .withExposedPorts(SOPHO_CONTAINER_PORT)
    .withWaitStrategy(
      Wait.forHttp(HEALTH_PATH, SOPHO_CONTAINER_PORT).forStatusCode(200),
    )
    .withStartupTimeout(SOPHO_CONTAINER_STARTUP_TIMEOUT_MILLISECONDS)
    .withLogConsumer((stream) => {
      stream.on("data", (chunk: Buffer | string) => {
        logs.push(chunk.toString());
      });
    });
  if (network !== null) {
    container = container.withNetwork(network);
  }
  if (dataVolumeName !== null) {
    container = container.withBindMounts([
      { source: dataVolumeName, target: SOPHO_CONTAINER_DATA_DIRECTORY },
    ]);
  }
  const startedContainer = await container.start();
  return {
    baseUrl: `http://${startedContainer.getHost()}:${startedContainer.getMappedPort(SOPHO_CONTAINER_PORT)}`,
    logs,
    container: startedContainer,
  };
}

export async function startMetadataPostgres(): Promise<MetadataPostgres> {
  const network = await new Network().start();
  const postgres = await new PostgreSqlContainer(POSTGRES_IMAGE)
    .withDatabase(METADATA_POSTGRES_DATABASE)
    .withUsername(METADATA_POSTGRES_USERNAME)
    .withPassword(METADATA_POSTGRES_PASSWORD)
    .withNetwork(network)
    .withNetworkAliases(METADATA_POSTGRES_ALIAS)
    .start();
  return { network, postgres };
}

export async function stopMetadataPostgres({
  network,
  postgres,
}: MetadataPostgres): Promise<void> {
  await postgres.stop();
  await network.stop();
}

export async function createDataVolume(volumeName: string): Promise<void> {
  const runtimeClient = await getContainerRuntimeClient();
  await runtimeClient.container.dockerode.createVolume({ Name: volumeName });
}

export async function removeDataVolume(volumeName: string): Promise<void> {
  const runtimeClient = await getContainerRuntimeClient();
  await runtimeClient.container.dockerode.getVolume(volumeName).remove();
}
