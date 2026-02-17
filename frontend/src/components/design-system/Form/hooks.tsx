import { createFormHook, createFormHookContexts } from "@tanstack/react-form";
import { TextField } from "./TextField";
import { SelectField } from "./SelectField";
import { SubscribeButton } from "./SubscribeButton";

export const { fieldContext, formContext, useFieldContext } =
  createFormHookContexts();

const formHook = createFormHook({
  fieldContext,
  formContext,
  fieldComponents: {
    TextField,
    SelectField,
    // NumberField,
  },
  formComponents: {
    SubscribeButton,
  },
});

export const { useAppForm, withForm } = formHook;
export type AppFormReturnType = ReturnType<typeof useAppForm>;
