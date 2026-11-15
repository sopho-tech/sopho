import { BannerSlim, Flex, Spinner } from "src/components/design-system";

export function UsageLoading() {
  return (
    <Flex
      justifyContent="center"
      paddingY="lg"
      role="status"
      aria-label="Loading AI usage"
    >
      <Spinner />
    </Flex>
  );
}

type UsageFailureProps = {
  failure: Error;
};

export function UsageFailure({ failure }: UsageFailureProps) {
  return <BannerSlim type="error" message={failure.message} />;
}
