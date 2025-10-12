import { useConnectionsStore } from "src/components/Connection/store";
import CreateNewConnectionStyles from "src/components/Connection/CreateNewConnection/CreateNewConnection.module.css";
import { Fragment, useEffect, useRef, useState } from "react";
import * as Form from "@radix-ui/react-form";
import * as Select from "@radix-ui/react-select";
import { NewAssetButton } from "src/components/NewAssetButton/NewAssetButton";
import { CloseButton } from "src/components/CloseButton/CloseButton";
import {
  ConnectionDetailsPageStateEnum,
  StatusType,
} from "src/components/Connection/dto";
import {
  useCreateConnection,
  CreateConnectionDto,
} from "src/api/connection/queries";

const CONNECTOR_DISPLAY_TO_BACKEND_VALUE_MAP: Record<string, string> = {
  PostgreSQL: "POSTGRESQL",
  MySQL: "MYSQL",
  Supabase: "POSTGRESQL",
};

const getDisplayNameFromBackendValue = (
  backendValue: string
): string | undefined => {
  return Object.keys(CONNECTOR_DISPLAY_TO_BACKEND_VALUE_MAP).find(
    (key) => CONNECTOR_DISPLAY_TO_BACKEND_VALUE_MAP[key] === backendValue
  );
};

const initialFormState: CreateConnectionDto = {
  source_type: CONNECTOR_DISPLAY_TO_BACKEND_VALUE_MAP["PostgreSQL"],
  name: "",
  username: "",
  password: "",
  host: "",
  database: "",
  description: null,
  schema: null,
  port: 0,
  status: StatusType.Active,
};

export function CreateNewConnection() {
  const { setConnectionDetailsPageState } = useConnectionsStore();
  const modalRef = useRef<HTMLDivElement>(null);
  const createMutation = useCreateConnection();

  const [formData, setFormData] = useState(initialFormState);
  const [currentPage, setCurrentPage] = useState(1);

  const handleInputChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>
  ) => {
    const { name, value } = e.target;

    if (name === "port") {
      const digitsOnly = value.replace(/[^0-9]/g, "");
      const newPortValue = digitsOnly === "" ? 0 : parseInt(digitsOnly, 10);
      setFormData((prev) => ({ ...prev, port: newPortValue }));
    } else {
      setFormData((prev) => ({ ...prev, [name]: value }));
    }
  };

  const goToNextPage = () => setCurrentPage(2);
  const goToPreviousPage = (e?: React.MouseEvent) => {
    if (e) e.preventDefault();
    setCurrentPage(1);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    createMutation.mutate(formData);
    setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST);
  };

  useEffect(() => {
    if (modalRef.current) modalRef.current.focus();

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape")
        setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST);
    };

    document.body.style.overflow = "hidden";
    window.addEventListener("keydown", handleKeyDown);

    return () => {
      document.body.style.overflow = "";
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [setConnectionDetailsPageState]);

  const renderConnectionFields = () => (
    <>
      <div className={CreateNewConnectionStyles.formRow}>
        <div className={CreateNewConnectionStyles.formLabel}>
          Selected Connector
        </div>
        <div className={CreateNewConnectionStyles.selectedConnector}>
          {getDisplayNameFromBackendValue(formData.source_type) ||
            formData.source_type}
        </div>
      </div>

      {renderFormField(
        "name",
        "Connector Name",
        "text",
        "My Database Connection"
      )}
      {renderFormField("username", "User", "text", "username")}
      {renderFormField("password", "Password", "password", "••••••••")}
      {renderFormField("host", "Host", "text", "localhost or db.example.com")}
      {renderFormField("port", "Port", "number", "5432")}
      {renderFormField("database", "Database", "text", "my_database")}
      {renderFormField("schema", "Schema (Optional)", "text", "public", false)}
      {renderFormField(
        "description",
        "Description (Optional)",
        "text",
        "A brief description of the connection",
        false
      )}
    </>
  );

  const renderFormField = (
    name: string,
    label: string,
    type: string,
    placeholder: string,
    isRequired: boolean = true
  ) => (
    <Form.Field className={CreateNewConnectionStyles.formRow} name={name}>
      <Form.Label className={CreateNewConnectionStyles.formLabel}>
        {label}
      </Form.Label>
      <Form.Control asChild>
        <input
          type={type}
          id={name}
          name={name}
          value={formData[name as keyof typeof formData] ?? ""}
          onChange={handleInputChange}
          required={isRequired}
          className={CreateNewConnectionStyles.formInput}
          placeholder={placeholder}
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

  return (
    <div
      className={CreateNewConnectionStyles.modalOverlay}
      onClick={() =>
        setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST)
      }
    >
      <div
        ref={modalRef}
        className={CreateNewConnectionStyles.connectionDetailsContainer}
        onClick={(e) => e.stopPropagation()}
        tabIndex={-1}
        role="dialog"
        aria-modal="true"
        aria-labelledby="connection-details-title"
      >
        <div className={CreateNewConnectionStyles.modalHeader}>
          <h1
            id="connection-details-title"
            className={CreateNewConnectionStyles.connectionDetailsTitle}
          >
            Connection Details
          </h1>
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
            {currentPage === 1 ? (
              <Form.Field
                className={CreateNewConnectionStyles.formRow}
                name="source_type"
              >
                <Form.Label className={CreateNewConnectionStyles.formLabel}>
                  Connector Type
                </Form.Label>
                <Form.Control asChild>
                  <Select.Root
                    name="source_type"
                    value={
                      getDisplayNameFromBackendValue(formData.source_type) || ""
                    }
                    onValueChange={(selectedDisplayName) => {
                      const backendValue =
                        CONNECTOR_DISPLAY_TO_BACKEND_VALUE_MAP[
                          selectedDisplayName
                        ];
                      if (backendValue) {
                        setFormData((prev) => ({
                          ...prev,
                          source_type: backendValue,
                        }));
                      }
                    }}
                  >
                    <Select.Trigger
                      className={CreateNewConnectionStyles.formInput}
                    >
                      <Select.Value placeholder="Select a database type" />
                    </Select.Trigger>
                    <Select.Portal>
                      <Select.Content
                        className={CreateNewConnectionStyles.selectContent}
                        position="popper"
                      >
                        <Select.Viewport>
                          {Object.keys(
                            CONNECTOR_DISPLAY_TO_BACKEND_VALUE_MAP
                          ).map((displayType) => (
                            <Select.Item
                              key={displayType}
                              value={displayType}
                              className={CreateNewConnectionStyles.selectItem}
                            >
                              <Select.ItemText>{displayType}</Select.ItemText>
                            </Select.Item>
                          ))}
                        </Select.Viewport>
                      </Select.Content>
                    </Select.Portal>
                  </Select.Root>
                </Form.Control>
              </Form.Field>
            ) : (
              renderConnectionFields()
            )}
          </div>

          <div className={CreateNewConnectionStyles.formActions}>
            {currentPage === 1 ? (
              <>
                <NewAssetButton
                  buttonText="Cancel"
                  onClick={() =>
                    setConnectionDetailsPageState(
                      ConnectionDetailsPageStateEnum.LIST
                    )
                  }
                />
                <NewAssetButton buttonText="Next" onClick={goToNextPage} />
              </>
            ) : (
              <Fragment>
                <NewAssetButton buttonText="Back" onClick={goToPreviousPage} />
                <Form.Submit asChild>
                  <NewAssetButton buttonText="Save" />
                </Form.Submit>
              </Fragment>
            )}
          </div>
        </Form.Root>
      </div>
    </div>
  );
}
