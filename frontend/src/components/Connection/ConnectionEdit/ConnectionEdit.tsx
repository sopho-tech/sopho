import { useConnectionsStore } from "src/components/Connection/store";
import { useState, useEffect, FormEvent } from "react";
import * as Form from "@radix-ui/react-form";
import CreateNewConnectionStyles from "src/components/Connection/CreateNewConnection/CreateNewConnection.module.css";
import { NewAssetButton } from "src/components/NewAssetButton/NewAssetButton";
import { CloseButton } from "../../CloseButton/CloseButton";
import {
  ConnectionDto,
  CONNECTOR_DISPLAY_TO_BACKEND_VALUE_MAP,
  ConnectionDetailsPageStateEnum,
} from "src/components/Connection/dto";
import { useConnection, useUpdateConnection } from "src/api/connection/queries";

const getDisplayNameFromBackendValue = (
  backendValue: string,
): string | undefined => {
  return Object.keys(CONNECTOR_DISPLAY_TO_BACKEND_VALUE_MAP).find(
    (key) => CONNECTOR_DISPLAY_TO_BACKEND_VALUE_MAP[key] === backendValue,
  );
};

const initialFormState: ConnectionDto | null = null;

export function ConnectionEdit() {
  const { connectionId, setConnectionDetailsPageState } = useConnectionsStore();
  const {
    data: connection,
    isLoading,
    error: queryError,
  } = useConnection(connectionId);
  const updateMutation = useUpdateConnection();

  const [formData, setFormData] = useState<ConnectionDto | null>(
    initialFormState,
  );

  useEffect(() => {
    // Only set formData if connection data is available AND formData hasn't been populated yet (is null).
    // This prevents overwriting user edits if the 'connection' object reference changes due to background refetches.
    if (connection && formData === null) {
      setFormData({
        ...connection, // Spread the fetched connection object
        // Ensure port is correctly formatted as number or null
        port:
          connection.port !== undefined &&
          connection.port !== null &&
          !isNaN(Number(connection.port))
            ? Number(connection.port)
            : null,
      });
    }
  }, [connection, formData]);

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setFormData((prevFormData) => {
      // If formData is not yet initialized by useEffect, prevFormData will be null.
      // In this case, we should not try to update it from here, as useEffect will set it.
      // Returning prevFormData (which is null) allows useEffect to proceed correctly.
      if (!prevFormData) {
        console.warn(
          "handleInputChange called before formData was initialized from connection. Input for this event may be ignored until form is fully ready.",
        );
        return null; // Or return prevFormData;
      }

      // If prevFormData is populated, create the new state.
      // Create a new object to ensure state update
      const updatedData = { ...prevFormData };

      if (name === "port") {
        const digitsOnly = value.replace(/[^0-9]/g, "");
        // Ensure port is number or null
        updatedData.port = digitsOnly === "" ? null : parseInt(digitsOnly, 10);
      } else {
        // For other fields, assign the value.
        // Type assertion needed if 'name' can be a key for non-string/number values
        // or if Connection type is stricter. For current form fields, this should be okay.
        (updatedData as any)[name] = value;
      }
      return updatedData;
    });
  };

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (!formData) {
      console.error("handleSubmit: formData is null. Cannot submit.");
      return;
    }

    const dataToSubmit: Partial<ConnectionDto> = { ...formData };

    updateMutation.mutate({ connectionId, payload: dataToSubmit });
  };

  const renderFormField = (
    name: keyof ConnectionDto,
    label: string,
    type: string,
    placeholder: string,
    isRequired: boolean = true,
  ) => {
    if (!formData) return null; // Ensure formData is available

    // Explicitly type value to avoid implicit any
    const fieldValue = formData[name] as string | number | null | undefined;

    return (
      <Form.Field
        className={CreateNewConnectionStyles.formRow}
        name={name as string}
      >
        <Form.Label className={CreateNewConnectionStyles.formLabel}>
          {label}
        </Form.Label>
        <Form.Control asChild>
          <input
            type={type}
            id={name as string}
            name={name as string}
            value={fieldValue ?? ""} // Use the typed fieldValue
            onChange={handleInputChange}
            required={isRequired}
            className={CreateNewConnectionStyles.formInput}
            placeholder={placeholder}
            autoComplete={"off"}
          />
        </Form.Control>
        {isRequired && (
          <Form.Message
            className={CreateNewConnectionStyles.errorMessage}
            match="valueMissing"
          >
            Please enter a {label.toLowerCase().replace(" (optional)", "")}
          </Form.Message>
        )}
      </Form.Field>
    );
  };

  if (isLoading) return <p>Loading connection details...</p>;
  if (queryError) return <p>Error loading connection: {queryError.message}</p>;
  if (!connection) return <p>No connection data found.</p>;

  return (
    <div className={CreateNewConnectionStyles.modalOverlay}>
      <div
        className={CreateNewConnectionStyles.connectionDetailsContainer}
        role="dialog"
        aria-modal="true"
        aria-labelledby="connection-edit-title"
      >
        <div className={CreateNewConnectionStyles.modalHeader}>
          <h4>Edit Connection</h4>
          <CloseButton
            onClick={() =>
              setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST)
            }
          />
        </div>

        <Form.Root
          onSubmit={handleSubmit}
          className={CreateNewConnectionStyles.connectionForm}
        >
          <div className={CreateNewConnectionStyles.formContent}>
            <div className={CreateNewConnectionStyles.formRow}>
              <div className={CreateNewConnectionStyles.formLabel}>
                Connector Type
              </div>
              <div className={CreateNewConnectionStyles.selectedConnector}>
                {formData &&
                  (getDisplayNameFromBackendValue(formData.source_type) ||
                    formData.source_type)}
              </div>
            </div>

            {renderFormField(
              "name",
              "Connector Name",
              "text",
              "My Database Connection",
            )}
            {renderFormField("username", "User", "text", "username")}
            {renderFormField(
              "password",
              "Password",
              "password",
              "*****",
              false,
            )}
            {renderFormField(
              "host",
              "Host",
              "text",
              "localhost or db.example.com",
            )}
            {renderFormField("port", "Port", "text", "5432")}
            {renderFormField("database", "Database", "text", "my_database")}
            {renderFormField(
              "schema",
              "Schema (Optional)",
              "text",
              "public",
              false,
            )}
            {renderFormField(
              "description",
              "Description (Optional)",
              "text",
              "A brief description of the connection",
              false,
            )}
          </div>

          <div className={CreateNewConnectionStyles.formActions}>
            <NewAssetButton
              buttonText="Cancel"
              onClick={
                () =>
                  setConnectionDetailsPageState &&
                  setConnectionDetailsPageState(
                    "LIST" as any,
                  ) /* Cast to any if enum causes issues here, or import enum */
              }
            />
            <Form.Submit asChild>
              <NewAssetButton
                buttonText="Save Changes"
                isLoading={updateMutation.isPending}
              />
            </Form.Submit>
          </div>
          {updateMutation.isError && (
            <p
              className={CreateNewConnectionStyles.errorMessage}
              style={{ textAlign: "center", marginTop: "10px" }}
            >
              Error:{" "}
              {updateMutation.error?.message || "Failed to update connection."}
            </p>
          )}
        </Form.Root>
      </div>
    </div>
  );
}
