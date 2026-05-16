import {
  Button,
  Flex,
  Form,
  Grid,
  GridItem,
  Heading,
  Separator,
} from "src/components/design-system";
import { SourceTypeEnum } from "src/constants/database_types";
import styles from "./ConnectionEdit.module.css";
import { useParams, useNavigate } from "react-router";
import { useConnection, useUpdateConnection } from "src/api/connection/queries";
import { useStore } from "src/store";
import { APP_ROUTES } from "src/constants/app_routes";
import { ConnectionDetailsPageStateEnum } from "../dto";

export function ConnectionEdit() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const updateMutation = useUpdateConnection();
  const setConnectionDetailsPageState = useStore(
    (state) => state.connection.setConnectionDetailsPageState
  );
  const {
    data: connection,
    isLoading,
    error: queryError,
  } = useConnection(id ?? "");

  const onSubmitHandler = (formData: FormData) => {
    if (!connection) return;
    const payload = {
      id: connection.id,
      source_type: connection.source_type,
      created_at: connection.created_at,
      updated_at: connection.updated_at,
      status: connection.status,
      name: formData.get("name") as string,
      host: formData.get("host") as string,
      port: formData.get("port") ? Number(formData.get("port")) : null,
      database: formData.get("database") as string,
      schema: (formData.get("schema") as string) || null,
      username: formData.get("username") as string,
      password: formData.get("password") as string,
      description: (formData.get("description") as string) || null,
    };
    updateMutation.mutate({ connectionId: connection.id, payload });
    setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST);
    navigate(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS);
  };

  const handleBack = () => {
    setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST);
    navigate(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS);
  };

  if (isLoading) return <div>Loading connection details...</div>;
  if (queryError)
    return <div>Error loading connection: {queryError.message}</div>;
  if (!connection || !id) return <div>No connection data found.</div>;

  const defaultValues: Record<string, unknown> = {
    name: connection.name,
    description: connection.description ?? "",
    username: connection.username,
    password: connection.password,
    host: connection.host,
    port: connection.port != null ? String(connection.port) : "",
    database: connection.database,
    schema: connection.schema ?? "",
  };

  const sourceTypeLabel =
    connection.source_type.charAt(0) +
    connection.source_type.slice(1).toLowerCase();

  return (
    <Flex
      flex="grow"
      paddingX="2xl"
      paddingY="xs"
      marginTop="xs"
      marginBottom="xs"
      marginLeft="xs"
      marginRight="xs"
      direction="column"
    >
      <Flex justifyContent="space-between">
        <Heading accessbilityLevel={1}>Edit Connection</Heading>
        <Button
          label="Go back to Settings"
          size="md"
          backgroundColor="accent"
          shape="rectangle"
          leadingIconName="chevron_left"
          onClick={handleBack}
        />
      </Flex>
      <Form
        onSubmit={onSubmitHandler}
        defaultValues={defaultValues}
        className={styles.form}
      >
        <Form.Fields className={styles.formFields}>
          <Flex direction="row" gap="lg" justifyContent="space-between">
            <Flex direction="column" gap="sm" width={"100%"}>
              <Heading accessbilityLevel={2} textColor="black" size="lg">
                Basic Details
              </Heading>
              <Heading accessbilityLevel={3} textColor="subtle" size="base">
                Set your basic connection details
              </Heading>
            </Flex>
            <Grid
              columnGutter="2xl"
              rowGutter="md"
              className={styles.basicFormFields}
            >
              <GridItem colSpan={6}>
                <Form.Field
                  name="name"
                  required
                  errorMessage="Enter the connector name"
                  className={styles.formField}
                >
                  <Form.Label className={styles.formLabel}>Name</Form.Label>
                  <Form.Input placeholder="Connection name" />
                </Form.Field>
              </GridItem>
              <GridItem colSpan={6}>
                <Form.Field name="description" className={styles.formField}>
                  <Form.Label className={styles.formLabel}>
                    Description
                  </Form.Label>
                  <Form.Input placeholder="Connection description" />
                </Form.Field>
              </GridItem>
              <GridItem colSpan={12}>
                <div className={styles.formField}>
                  <Form.Label className={styles.formLabel}>
                    Source Type
                  </Form.Label>
                  <div
                    style={{
                      padding: "var(--space-sm) var(--space-md)",
                      backgroundColor: "var(--color-grey-100)",
                      borderRadius: "var(--radius-md)",
                      color: "var(--color-grey-600)",
                    }}
                  >
                    {sourceTypeLabel}
                  </div>
                </div>
              </GridItem>
            </Grid>
          </Flex>
          <Separator />
          <Flex direction="row" gap="lg" justifyContent="space-between">
            <Flex direction="column" gap="sm" width={"100%"}>
              <Heading accessbilityLevel={2} textColor="black" size="lg">
                Specific Details
              </Heading>
              <Heading accessbilityLevel={3} textColor="subtle" size="base">
                {connection.source_type === SourceTypeEnum.PostgreSQL
                  ? "Enter your PostgreSQL connection details"
                  : connection.source_type === SourceTypeEnum.Supabase
                    ? "Enter your Supabase connection details"
                    : `Enter your ${sourceTypeLabel} connection details`}
              </Heading>
            </Flex>
            {(connection.source_type === SourceTypeEnum.PostgreSQL ||
              connection.source_type === SourceTypeEnum.Supabase) && (
              <Grid
                columnGutter="2xl"
                rowGutter="md"
                className={styles.basicFormFields}
              >
                <GridItem colSpan={6}>
                  <Form.Field
                    name="username"
                    required
                    errorMessage="Enter the username"
                    className={styles.formField}
                  >
                    <Form.Label className={styles.formLabel}>
                      Username
                    </Form.Label>
                    <Form.Input placeholder="admin" />
                  </Form.Field>
                </GridItem>
                <GridItem colSpan={6}>
                  <Form.Field
                    name="password"
                    required
                    errorMessage="Enter the password"
                    className={styles.formField}
                  >
                    <Form.Label className={styles.formLabel}>
                      Password
                    </Form.Label>
                    <Form.Password placeholder="toughpassword@123" />
                  </Form.Field>
                </GridItem>
                <GridItem colSpan={6}>
                  <Form.Field
                    name="host"
                    required
                    errorMessage="Enter the database host name"
                    className={styles.formField}
                  >
                    <Form.Label className={styles.formLabel}>Host</Form.Label>
                    <Form.Input placeholder="localhost" />
                  </Form.Field>
                </GridItem>
                <GridItem colSpan={6}>
                  <Form.Field
                    name="port"
                    required
                    errorMessage="Enter the database port number"
                    className={styles.formField}
                  >
                    <Form.Label className={styles.formLabel}>Port</Form.Label>
                    <Form.Input placeholder="5432" />
                  </Form.Field>
                </GridItem>
                <GridItem colSpan={6}>
                  <Form.Field
                    name="database"
                    required
                    errorMessage="Enter the database name"
                    className={styles.formField}
                  >
                    <Form.Label className={styles.formLabel}>
                      Database
                    </Form.Label>
                    <Form.Input placeholder="my_database" />
                  </Form.Field>
                </GridItem>
                <GridItem colSpan={6}>
                  <Form.Field
                    name="schema"
                    errorMessage="Enter the schema"
                    className={styles.formField}
                  >
                    <Form.Label className={styles.formLabel}>
                      Schema (optional)
                    </Form.Label>
                    <Form.Input placeholder="public" />
                  </Form.Field>
                </GridItem>
              </Grid>
            )}
            {connection.source_type === SourceTypeEnum.Sqlite && (
              <Grid
                columnGutter="2xl"
                rowGutter="2xl"
                className={styles.basicFormFields}
              >
                <GridItem colSpan={6}>
                  <Form.Field
                    name="database"
                    required
                    errorMessage="Invalid path to the database file"
                    className={styles.formField}
                  >
                    <Form.Label className={styles.formLabel}>
                      Path to the database file
                    </Form.Label>
                    <Form.Input placeholder="/Users/username/database.db" />
                  </Form.Field>
                </GridItem>
              </Grid>
            )}
          </Flex>
        </Form.Fields>
        <Separator />
        <Flex gap="md">
          <Form.Submit label="Save Changes" size="md" />
          <Button
            backgroundColor="white"
            label="Back"
            shape="rectangle"
            size="md"
            onClick={handleBack}
          />
        </Flex>
      </Form>
    </Flex>
  );
}
