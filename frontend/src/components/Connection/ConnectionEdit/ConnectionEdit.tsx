import { useConnectionsStore } from "src/components/Connection/store";
import { ConnectionDetailsPageStateEnum } from "src/components/Connection/dto";
import {
  SophoForm,
  SophoFormElement,
  SophoFormElementType,
  InputType,
} from "src/components/SophoForm";
import { SourceTypeEnum } from "src/constants/database_types";
import { SophoDialog } from "src/components/SophoDialog";
import { useConnection, useUpdateConnection } from "src/api/connection/queries";
import ConnectionEditStyles from "src/components/Connection/ConnectionEdit/ConnectionEdit.module.css";

const sourceTypeOptions = Object.values(SourceTypeEnum).map((type) => ({
  value: type,
  label: type.charAt(0) + type.slice(1).toLowerCase(),
}));

export function ConnectionEdit() {
  const updateMutation = useUpdateConnection();
  const {
    connectionId,
    connectionDetailsPageState,
    setConnectionDetailsPageState,
  } = useConnectionsStore();
  const {
    data: connection,
    isLoading,
    error: queryError,
  } = useConnection(connectionId);

  const shouldOpenDialog =
    connectionDetailsPageState === ConnectionDetailsPageStateEnum.EDIT;

  function handleOnOpenChange(open: boolean) {
    if (!open) {
      setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST);
    }
  }

  function handleDialogClose() {
    setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST);
  }

  function handleCancelCallback() {
    setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST);
  }

  function handleSubmitCallback(formData: FormData) {
    if (!connection) {
      throw new Error("Connection is null in handleSubmitCallback");
    }
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
    updateMutation.mutate({ connectionId, payload });
    setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST);
  }

  if (isLoading) return <div>Loading connection details...</div>;
  if (queryError)
    return <div>Error loading connection: {queryError.message}</div>;
  if (!connection) return <div>No connection data found.</div>;

  const formElements: SophoFormElement[] = [
    {
      key: "source_type",
      name: "Source Type",
      required: true,
      error_message: "Source is not selected",
      type: SophoFormElementType.SELECT,
      options: sourceTypeOptions,
      defaultValue: connection.source_type,
      disabled: true,
    },
    {
      key: "name",
      name: "Connector Name",
      required: true,
      error_message: "Enter the connector name",
      placeholder: "e.g., my connector",
      type: SophoFormElementType.INPUT,
      input_type: InputType.TEXT,
      defaultValue: connection.name,
    },
    {
      key: "username",
      name: "Username",
      required: true,
      error_message: "Enter the username",
      placeholder: "e.g., admin",
      type: SophoFormElementType.INPUT,
      input_type: InputType.TEXT,
      defaultValue: connection.username,
    },
    {
      key: "password",
      name: "Password",
      required: false,
      error_message: "Enter the password",
      placeholder: "*****",
      type: SophoFormElementType.INPUT,
      input_type: InputType.PASSWORD,
      defaultValue: connection.password,
    },
    {
      key: "host",
      name: "Host",
      required: true,
      error_message: "Enter the database host name",
      placeholder: "e.g., localhost or db.example.com",
      type: SophoFormElementType.INPUT,
      input_type: InputType.TEXT,
      defaultValue: connection.host,
    },
    {
      key: "port",
      name: "Port",
      required: true,
      error_message: "Enter the database port number",
      placeholder: "e.g., 5432",
      type: SophoFormElementType.INPUT,
      input_type: InputType.NUMBER,
      defaultValue: connection.port,
    },
    {
      key: "database",
      name: "Database",
      required: true,
      error_message: "Enter the database name",
      placeholder: "my_database",
      type: SophoFormElementType.INPUT,
      input_type: InputType.TEXT,
      defaultValue: connection.database,
    },
    {
      key: "schema",
      name: "Schema (optional)",
      required: false,
      error_message: "Enter the schema",
      placeholder: "public",
      type: SophoFormElementType.INPUT,
      input_type: InputType.TEXT,
      defaultValue: connection.schema,
    },
    {
      key: "description",
      name: "Description (optional)",
      required: false,
      error_message: "Enter a description",
      placeholder: "A brief description of the connection",
      type: SophoFormElementType.INPUT,
      input_type: InputType.TEXT,
      defaultValue: connection.description,
    },
  ];

  const dialogContent = (
    <SophoForm
      formElements={formElements}
      onSubmitCallback={handleSubmitCallback}
      onCancelCallback={handleCancelCallback}
      submitButtonText="Save Changes"
      showCancelButton={true}
      showSubmitButton={true}
    />
  );

  return (
    <SophoDialog
      shouldOpenDialog={shouldOpenDialog}
      handleOnOpenChange={handleOnOpenChange}
      handleDialogClose={handleDialogClose}
      info={dialogContent}
      title="Edit Connection"
      description="Update your database connection details"
    />
  );
}
