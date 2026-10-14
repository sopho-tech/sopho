import { createFormHook, createFormHookContexts } from "@tanstack/react-form";
import { TextField } from "src/components/design-system/Form/TextField";
import { SelectField } from "src/components/design-system/Form/SelectField";
import { SubscribeButton } from "src/components/design-system/Form/SubscribeButton";

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
