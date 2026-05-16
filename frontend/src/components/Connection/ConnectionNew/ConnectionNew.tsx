import {
  Button,
  Flex,
  Form,
  Grid,
  GridItem,
  Heading,
  Separator,
  Text,
} from "src/components/design-system";
import { SourceTypeEnum } from "src/constants/database_types";
import styles from "./ConnectionNew.module.css";
import { useState } from "react";
import { CreateConnectionDto, useCreateConnection } from "src/api/connection";
import { ConnectionDetailsPageStateEnum, StatusType } from "../dto";
import { useStore } from "src/store";
import { useNavigate } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";

export function ConnectionNew() {
  const [sourceType, setSourceType] = useState<string | null>(null);
  const createMutation = useCreateConnection();
  const setConnectionDetailsPageState = useStore(
    (state) => state.connection.setConnectionDetailsPageState
  );
  const navigate = useNavigate();
  const sourceTypeOptions = Object.values(SourceTypeEnum).map((type) => ({
    value: type,
    label: type.charAt(0) + type.slice(1).toLowerCase(),
  }));

  const onSubmitHandler = (formData: FormData) => {
    const payload: CreateConnectionDto = {
      name: formData.get("name") as string,
      source_type: formData.get("source_type") as string,
      host: formData.get("host") as string,
      port: formData.get("port") ? Number(formData.get("port")) : null,
      database: formData.get("database") as string,
      schema: (formData.get("schema") as string) || null,
      username: formData.get("username") as string,
      password: formData.get("password") as string,
      description: (formData.get("description") as string) || null,
      status: StatusType.Active,
    };
    createMutation.mutate(payload);
    setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST);
    navigate(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS);
  };

  const onChangeHandler = (_: FormData, fieldName: string, value: string) => {
    if (fieldName === "source_type") {
      setSourceType(value);
    }
  };

  const handleBack = () => {
    setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST);
    navigate(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS);
  };

  return (
    <Flex flex="grow" direction="column" gap="lg" width="100%">
      <Flex
        direction="row"
        justifyContent="space-between"
        alignItems="flex-start"
        gap="md"
      >
        <Flex direction="column" gap="2xs">
          <Heading accessbilityLevel={2} size="2xl" textColor="black">
            New Connection
          </Heading>
          <Text fontSize="sm" color="subtle">
            Connect a new data source to your workspace.
          </Text>
        </Flex>
        <Button
          label="Go back to Settings"
          size="md"
          backgroundColor="accent"
          shape="rectangle"
          leadingIconName="chevron_left"
          onClick={handleBack}
        ></Button>
      </Flex>
      <Form
        onSubmit={onSubmitHandler}
        onChange={onChangeHandler}
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
                <Form.Field name="name" className={styles.formField}>
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
                <Form.Field name="source_type" className={styles.formField}>
                  <Form.Label className={styles.formLabel}>
                    Source Type
                  </Form.Label>
                  <Form.Select
                    options={sourceTypeOptions}
                    groupName="Source Type"
                  />
                </Form.Field>
              </GridItem>
            </Grid>
          </Flex>
          <Separator></Separator>
          <Flex direction="row" gap="lg" justifyContent="space-between">
            <Flex direction="column" gap="sm" width={"100%"}>
              <Heading accessbilityLevel={2} textColor="black" size="lg">
                Specific Details
              </Heading>
              <Heading accessbilityLevel={3} textColor="subtle" size="base">
                {sourceType === null
                  ? "Select source type to see the specific details"
                  : sourceType === SourceTypeEnum.PostgreSQL
                    ? "Enter your PostgreSQL connection details"
                    : sourceType === SourceTypeEnum.Supabase
                      ? "Enter your Supabase connection details"
                      : `Enter your ${sourceType.charAt(0) + sourceType.slice(1).toLowerCase()} connection details`}
              </Heading>
            </Flex>
            {(sourceType === SourceTypeEnum.PostgreSQL ||
              sourceType === SourceTypeEnum.Supabase) && (
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
            {sourceType === SourceTypeEnum.Sqlite && (
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
        <Separator></Separator>
        <Flex gap="md">
          <Form.Submit label="Save" size="md" />
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
