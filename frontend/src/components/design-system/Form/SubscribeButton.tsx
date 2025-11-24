import { useFormContext } from "src/components/design-system/Form/form-context";
import { Button } from "src/components/design-system/Button";

export function SubscribeButton({ label }: { label: string }) {
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
          size="sm"
          shape="rectangle"
          type="submit"
          disabled={isSubmitting}
        />
      )}
    </form.Subscribe>
  );
}
