import { Fragment, useCallback, useMemo } from "react";
import { ConnectionDetailsPageStateEnum } from "src/components/Connection/dto";
import { useStore } from "src/store";
import { StatusBadge } from "src/components/StatusBadge/StatusBadge";
import { useConnection } from "src/api/connection/queries";
import { SophoDialog } from "src/components/SophoDialog/SophoDialog";
import { Form } from "src/components/design-system/Form/Form";
import { Grid, GridItem } from "src/components/design-system/Grid";
import { formatTimestamp } from "src/utils/timestamp_utils";
import styles from "./ConnectionDetail.module.css";

export function ConnectionDetail() {
  const connectionId = useStore((state) => state.connection.connectionId);
  const connectionDetailsPageState = useStore(
    (state) => state.connection.connectionDetailsPageState
  );
  const setConnectionDetailsPageState = useStore(
    (state) => state.connection.setConnectionDetailsPageState
  );

  const { data: connectionDetails } = useConnection(connectionId);

  const handleOpenChange = useCallback(
    (open: boolean) => {
      if (!open) {
        setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST);
      }
    },
    [setConnectionDetailsPageState]
  );

  const handleDialogClose = useCallback(() => {
    setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST);
  }, [setConnectionDetailsPageState]);

  const defaultValues = useMemo(
    () =>
      connectionDetails
        ? {
            name: connectionDetails.name,
            source_type:
              connectionDetails.source_type.charAt(0) +
              connectionDetails.source_type.slice(1).toLowerCase(),
            host: connectionDetails.host,
            port:
              connectionDetails.port != null
                ? String(connectionDetails.port)
                : "",
            username: "********",
            password: "********",
            database: connectionDetails.database,
            schema: connectionDetails.schema ?? "",
            description: connectionDetails.description ?? "",
            created_at: formatTimestamp(connectionDetails.created_at),
            updated_at: formatTimestamp(connectionDetails.updated_at),
          }
        : null,
    [connectionDetails]
  );

  const handleSubmit = useCallback(() => {}, []);

  const dialogContent = (
    <Fragment>
      {defaultValues && connectionDetails && (
        <Form defaultValues={defaultValues} onSubmit={handleSubmit} readonly>
          <Form.Fields className={styles.fieldsContainer}>
            <Grid columnGutter="2xl" rowGutter="md" className={styles.grid}>
              <GridItem colSpan={6}>
                <Form.Field name="name" className={styles.fieldContainer}>
                  <Form.Label>Name</Form.Label>
                  <Form.Input />
                </Form.Field>
              </GridItem>
              <GridItem colSpan={6}>
                <Form.Field
                  name="source_type"
                  className={styles.fieldContainer}
                >
                  <Form.Label>Source Type</Form.Label>
                  <Form.Input />
                </Form.Field>
              </GridItem>
              <GridItem colSpan={6}>
                <Form.Field name="host" className={styles.fieldContainer}>
                  <Form.Label>Host</Form.Label>
                  <Form.Input />
                </Form.Field>
              </GridItem>
              <GridItem colSpan={6}>
                <Form.Field name="port" className={styles.fieldContainer}>
                  <Form.Label>Port</Form.Label>
                  <Form.Input />
                </Form.Field>
              </GridItem>
              <GridItem colSpan={6}>
                <Form.Field name="username" className={styles.fieldContainer}>
                  <Form.Label>Username</Form.Label>
                  <Form.Input />
                </Form.Field>
              </GridItem>
              <GridItem colSpan={6}>
                <Form.Field name="password" className={styles.fieldContainer}>
                  <Form.Label>Password</Form.Label>
                  <Form.Password />
                </Form.Field>
              </GridItem>
              <GridItem colSpan={6}>
                <Form.Field name="database" className={styles.fieldContainer}>
                  <Form.Label>Database</Form.Label>
                  <Form.Input />
                </Form.Field>
              </GridItem>
              <GridItem colSpan={6}>
                <Form.Field name="schema" className={styles.fieldContainer}>
                  <Form.Label>Schema</Form.Label>
                  <Form.Input />
                </Form.Field>
              </GridItem>
              <GridItem colSpan={6}>
                <Form.Field
                  name="description"
                  className={styles.fieldContainer}
                >
                  <Form.Label>Description</Form.Label>
                  <Form.Input />
                </Form.Field>
              </GridItem>
              <GridItem colSpan={6}>
                <Form.Field name="created_at" className={styles.fieldContainer}>
                  <Form.Label>Created On</Form.Label>
                  <Form.Input />
                </Form.Field>
              </GridItem>
              <GridItem colSpan={6}>
                <Form.Field name="updated_at" className={styles.fieldContainer}>
                  <Form.Label>Last Modified</Form.Label>
                  <Form.Input />
                </Form.Field>
              </GridItem>
              <GridItem colSpan={6}>
                <div className={styles.fieldContainer}>
                  <Form.Label>Status</Form.Label>
                  <StatusBadge
                    status={connectionDetails.status}
                    text={connectionDetails.status}
                  />
                </div>
              </GridItem>
            </Grid>
          </Form.Fields>
        </Form>
      )}
    </Fragment>
  );

  return (
    <SophoDialog
      shouldOpenDialog={
        connectionDetailsPageState === ConnectionDetailsPageStateEnum.DETAIL
      }
      handleOnOpenChange={handleOpenChange}
      handleDialogClose={handleDialogClose}
      title="Connection Details"
      info={dialogContent}
    />
  );
}
