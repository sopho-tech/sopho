import { Flex, IconButton, SearchBar } from "src/components/design-system";
import { useNavigate } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import { TopBar } from "../TopBar";
import { Sidebar } from "src/components/ConversationalAnalytics/Sidebar";
import { ConversationSection } from "./ConversationSection";
import { ConversationStreamProvider } from "./ConversationStreamContext";

export function ConversationalAnalytics() {
  const navigate = useNavigate();

  const handleCreateConversation = () => {
    navigate(APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.INDEX);
  };

  return (
    <Flex direction="column" flex="grow" overflow="scrollY">
      <TopBar>
        <TopBar.Left>
          <Flex gap="sm">
            <IconButton
              type="add"
              backgroundColor="default"
              iconColor="default"
              tooltip={{ text: "New conversation" }}
              onClick={handleCreateConversation}
            />
          </Flex>
        </TopBar.Left>
        <TopBar.Center>
          <SearchBar />
        </TopBar.Center>
        <TopBar.Right></TopBar.Right>
      </TopBar>
      <ConversationStreamProvider>
        <Flex direction="row" gap="lg" flex="grow" overflow="hidden">
          <Sidebar />
          <ConversationSection />
        </Flex>
      </ConversationStreamProvider>
    </Flex>
  );
}
