import { Flex, Heading } from "src/components/design-system";
import {
  HomeHeader,
  HomeEmptyState,
  RecentlyUpdatedCanvases,
} from "./components";

export function Home() {
  return (
    <Flex direction="column" flex="grow" gap="lg" overflow="scrollY">
      <HomeHeader />
      <Flex paddingX="2xl" direction="column" gap="lg">
        <Heading accessbilityLevel={1}>Home</Heading>
        <RecentlyUpdatedCanvases />
        <HomeEmptyState />
      </Flex>
    </Flex>
  );
}
