import { Box, type BoxProps } from "src/components/design-system/Box/Box";

type FlexProps = Omit<BoxProps, "display">;

export function Flex(props: FlexProps) {
  return <Box display="flex" {...props} />;
}
