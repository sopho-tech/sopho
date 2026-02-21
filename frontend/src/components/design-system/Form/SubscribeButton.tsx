import { useFormContext } from "src/components/design-system/Form/form-context";
import { Button } from "src/components/design-system/Button";
import { ButtonSize } from "../datatypes";

export function SubscribeButton({
  label,
  size,
}: {
  label: string;
  size: ButtonSize;
}) {
  const form = useFormContext();
  return (
    <form.Subscribe selector={(state) => state.isSubmitting}>
      {(isSubmitting) => (
        <Button
          label={label}
          onClick={() => {
            form.handleSubmit();
          }}
          backgroundColor="accent"
          size={size}
          shape="rectangle"
          type="submit"
          disabled={isSubmitting}
        />
      )}
    </form.Subscribe>
  );
}
