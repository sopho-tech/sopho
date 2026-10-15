import { createContext, useContext } from "react";
import { createFormHookContexts } from "@tanstack/react-form";

export const { fieldContext, formContext, useFieldContext, useFormContext } =
  createFormHookContexts();

export const FormFieldIdContext = createContext<string | null>(null);

export function useFormFieldId(): string {
  const formFieldId = useContext(FormFieldIdContext);
  if (formFieldId === null) {
    throw new Error("Form inputs must be rendered inside a Form.Field");
  }
  return formFieldId;
}
