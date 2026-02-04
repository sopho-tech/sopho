import { Flex, Heading } from "src/components/design-system";
import { HomeHeader, RecentlyUpdatedCanvases } from "./components";

export function Home() {
  return (
    <Flex direction="column" flex="grow" gap="lg" overflow="scrollY">
      <HomeHeader />
      <Flex paddingX="lg" direction="column" gap="lg">
        <Heading accessbilityLevel={1}>Home</Heading>
        <RecentlyUpdatedCanvases />
      </Flex>
    </Flex>
  );
}
