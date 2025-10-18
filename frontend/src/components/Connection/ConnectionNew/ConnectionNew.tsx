import { useConnectionsStore } from "src/components/Connection/store";
import {
  ConnectionDetailsPageStateEnum,
  StatusType,
} from "src/components/Connection/dto";
import {
  SophoMultiStepForm,
  SophoMultiStepFormLayout,
  SophoMultiStepFormStep,
} from "src/components/SophoMultiStepForm";
import { InputType, SophoFormElementType } from "src/components/SophoForm";
import { SourceTypeEnum } from "src/constants/database_types";
import { SophoDialog } from "src/components/SophoDialog";
import { SophoProgress } from "src/components/SophoProgress";
import ConnectionNewStyles from "src/components/Connection/ConnectionNew/ConnectionNew.module.css";
import { useState } from "react";
import {
  useCreateConnection,
  CreateConnectionDto,
} from "src/api/connection/queries";

const sourceTypeOptions = Object.values(SourceTypeEnum).map((type) => ({
  value: type,
  label: type.charAt(0) + type.slice(1).toLowerCase(),
}));

const steps: SophoMultiStepFormStep[] = [
  {
    formElements: [
      {
        key: "source_type",
        name: "Source Type",
        required: true,
        error_message: "Source is not selected",
        type: SophoFormElementType.SELECT,
        options: sourceTypeOptions,
      },
    ],
    backButtonText: "Back",
    nextButtonText: "Next",
    showBackButton: true,
    showNextButton: true,
  },
  {
    formElements: [
      {
        key: "name",
        name: "Connector Name",
        required: true,
        error_message: "Enter the connector name",
        placeholder: "e.g., my connector",
        type: SophoFormElementType.INPUT,
        input_type: InputType.TEXT,
      },
      {
        key: "username",
        name: "Username",
        required: true,
        error_message: "Enter the username",
        placeholder: "e.g., admin",
        type: SophoFormElementType.INPUT,
        input_type: InputType.TEXT,
      },
      {
        key: "password",
        name: "Password",
        required: true,
        error_message: "Enter the username",
        placeholder: "e.g., toughpassword@123",
        type: SophoFormElementType.INPUT,
        input_type: InputType.PASSWORD,
      },
      {
        key: "host",
        name: "Host",
        required: true,
        error_message: "Enter the database host name",
        placeholder: "e.g., localhost or db.example.com",
        type: SophoFormElementType.INPUT,
        input_type: InputType.TEXT,
      },
      {
        key: "port",
        name: "Port",
        required: true,
        error_message: "Enter the database port number",
        placeholder: "e.g., 5432",
        type: SophoFormElementType.INPUT,
        input_type: InputType.NUMBER,
      },
      {
        key: "database",
        name: "Database",
        required: true,
        error_message: "Enter the database name",
        placeholder: "my_database",
        type: SophoFormElementType.INPUT,
        input_type: InputType.TEXT,
      },
      {
        key: "schema",
        name: "Schema (optional)",
        required: false,
        error_message: "Enter the schema",
        placeholder: "public",
        type: SophoFormElementType.INPUT,
        input_type: InputType.TEXT,
      },
      {
        key: "description",
        name: "Description (optional)",
        required: false,
        error_message: "Enter a description",
        placeholder: "A brief description of the connection",
        type: SophoFormElementType.INPUT,
        input_type: InputType.TEXT,
      },
    ],
    backButtonText: "Back",
    nextButtonText: "Submit",
    showBackButton: true,
    showNextButton: true,
  },
];

export function ConnectionNew() {
  const createMutation = useCreateConnection();
  const [completedSteps, setCompletedSteps] = useState(0);
  const { connectionDetailsPageState, setConnectionDetailsPageState } =
    useConnectionsStore();
  const shouldOpenDialog =
    connectionDetailsPageState == ConnectionDetailsPageStateEnum.NEW;
  const totalSteps = steps.length;

  function handleOnOpenChange(open: boolean) {
    if (!open) {
      setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST);
    }
  }

  function handleDialogClose() {
    setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST);
  }

  function handleSubmitCallback(formData: FormData) {
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
  }

  function handleProgressChange(newStepNumber: number) {
    setCompletedSteps(newStepNumber - 1);
  }

  const dialogContent = (
    <SophoMultiStepForm
      steps={steps}
      type={SophoMultiStepFormLayout.VERTICAL}
      formElementsStyleClass={ConnectionNewStyles.formElements}
      onSubmitCallback={handleSubmitCallback}
      onProgressChange={handleProgressChange}
    />
  );
  return (
    <div>
      <SophoDialog
        shouldOpenDialog={shouldOpenDialog}
        handleOnOpenChange={handleOnOpenChange}
        handleDialogClose={handleDialogClose}
        info={dialogContent}
        title="New Connection"
        description="Set up a new database connection"
        titleAccessory={
          <SophoProgress total={totalSteps} completed={completedSteps} />
        }
      />
    </div>
  );
}
