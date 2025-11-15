import { Box } from "src/components/design-system/Box/Box";
import type { SharedLayoutProps } from "src/components/design-system/datatypes";

type FlexProps = SharedLayoutProps & {
  children: React.ReactNode;
};

export function Flex(props: FlexProps) {
  return <Box display="flex" {...props} />;
}
