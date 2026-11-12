import { useFormContext } from "src/components/design-system/Form/form-context";
import { Button } from "src/components/design-system/Button";
import { ButtonSize } from "src/components/design-system/datatypes";

export function SubscribeButton({
  label,
  size,
  fullWidth = false,
  isPending = false,
}: {
  label: string;
  size: ButtonSize;
  fullWidth?: boolean;
  isPending?: boolean;
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
          disabled={isSubmitting || isPending}
          fullWidth={fullWidth}
          contentAlignment="center"
        />
      )}
    </form.Subscribe>
  );
}
