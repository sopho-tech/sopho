import { Flex } from "src/components/design-system";
import {
  HomeHeader,
  HomeEmptyState,
  RecentlyUpdatedCanvases,
} from "./components";

export function Home() {
  return (
    <Flex direction="column" flex="grow" overflow="hidden">
      <HomeHeader />
      <Flex
        paddingX="2xl"
        direction="column"
        gap="lg"
        flex="grow"
        overflow="scrollY"
      >
        <RecentlyUpdatedCanvases />
        <HomeEmptyState />
      </Flex>
    </Flex>
  );
}
