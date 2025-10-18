import { SophoFormElement } from "src/components/SophoForm";

export function mergeForms(
  currentFormData: FormData,
  incomingFormData: HTMLFormElement
): FormData {
  const mergedFormData = new FormData();

  for (const [key, value] of currentFormData.entries()) {
    mergedFormData.append(key, value);
  }

  const newFormData = new FormData(incomingFormData);
  for (const [key, value] of newFormData.entries()) {
    mergedFormData.delete(key);
    mergedFormData.append(key, value);
  }

  return mergedFormData;
}

export function getInitialFormValues(
  formElements: SophoFormElement[]
): FormData {
  const formData = new FormData();
  for (const formElement of formElements) {
    if (formElement.disabled === true && formElement.defaultValue != null) {
      formData.append(formElement.key, String(formElement.defaultValue));
    }
  }
  return formData;
}
