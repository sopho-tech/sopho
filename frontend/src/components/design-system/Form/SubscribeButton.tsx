import { useFormContext } from "src/components/design-system/Form/form-context";
import { Button } from "src/components/design-system/Button";
import { ButtonSize } from "src/components/design-system/datatypes";

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
            void form.handleSubmit();
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
