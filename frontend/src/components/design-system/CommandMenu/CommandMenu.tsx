// TODO Incorporate fix of https://github.com/dip/cmdk/issues/393 when released

import { Command } from "cmdk";
import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router";
import { Badge } from "src/components/design-system";
import { Flex } from "src/components/design-system";
import { SearchResultItemDto, useSearch } from "src/api/search/queries";
import { EntityType } from "src/api/search";
import { APP_ROUTES, SEARCH_PARAMS } from "src/constants/app_routes";
import {
  KEYBOARD_SHORTCUTS,
  useKeyboardShortcut,
} from "src/utils/keyboard_shortcuts";
import { useStore } from "src/store";
import { CanvasesPageState } from "src/components/Canvases/dto";
import { useDebouncedValue } from "src/utils/hooks";
import { CommandMenuHomePage } from "./CommandMenuHomePage.tsx";
import { CommandMenuSearchResults } from "./CommandMenuSearchResults.tsx";
import { Page } from "./CommandMenu.types.ts";

const RECENT_ITEMS_LIMIT = 5;

const PageToDisplayNameMap: Record<Page, string> = {
  home: "Home",
  canvases: "Canvases",
  sql_cells: "SQL Cells",
  chart_cells: "Chart Cells",
  conversations: "Conversations",
};

const PageToFiltersMap: Record<Page, EntityType[]> = {
  home: [
    EntityType.Canvas,
    EntityType.ChartCell,
    EntityType.SqlCell,
    EntityType.Conversation,
  ],
  canvases: [EntityType.Canvas],
  sql_cells: [EntityType.SqlCell],
  chart_cells: [EntityType.ChartCell],
  conversations: [EntityType.Conversation],
};

export const CommandMenu = () => {
  const navigate = useNavigate();
  const setCanvasPageState = useStore((state) => state.canvas.setCanvasPageState);
  const isCommandMenuOpen = useStore((state) => state.commandMenu.isOpen);
  const setIsCommandMenuOpen = useStore((state) => state.commandMenu.setIsOpen);
  const toggleIsCommandMenuOpen = useStore(
    (state) => state.commandMenu.toggleIsOpen
  );
  const [inputValue, setInputValue] = useState<string>("");
  const debouncedInputValue = useDebouncedValue(inputValue);
  const [pages, setPages] = useState<Page[]>(["home"]);
  const currentPage = pages[pages.length - 1];
  const isHomePage = currentPage === "home";
  const filters = PageToFiltersMap[currentPage];
  const searchQuery = useSearch(
    debouncedInputValue,
    filters,
    isCommandMenuOpen
  );

  useKeyboardShortcut(
    toggleIsCommandMenuOpen,
    KEYBOARD_SHORTCUTS.OPEN_COMMAND_MENU
  );

  useEffect(() => {
    if (isCommandMenuOpen) {
      setPages(["home"]);
      setInputValue("");
    }
  }, [isCommandMenuOpen]);

  const popPage = () => {
    setPages((prevPages) => {
      const nextPages = [...prevPages];
      nextPages.splice(-1, 1);
      return nextPages;
    });
  };

  const handlePageSelect = (page: Page) => {
    setPages((prevPages) => [...prevPages, page]);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      setInputValue("");
    }
    if (isHomePage || inputValue.length) {
      return;
    }
    if (e.key === "Backspace") {
      e.preventDefault();
      popPage();
    }
  };

  const filter = (value: string, search: string) => {
    if (inputValue !== "") {
      return 1;
    }
    const extendedSearch = search.toLowerCase();
    return value.toLowerCase().includes(extendedSearch) ? 1 : 0;
  };

  const handleSearchResultSelect = (item: SearchResultItemDto) => {
    if (item.entity_type === EntityType.Canvas) {
      setIsCommandMenuOpen(false);
      navigate(APP_ROUTES.CANVAS.replace(":id", item.id));
      return;
    }
    if (item.entity_type === EntityType.Conversation) {
      setIsCommandMenuOpen(false);
      navigate(
        APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.CONVERSATION.replace(
          ":id",
          item.id
        )
      );
      return;
    }
    if (!item.canvas_id) {
      return;
    }
    setIsCommandMenuOpen(false);
    navigate(
      `${APP_ROUTES.CANVAS.replace(":id", item.canvas_id)}?${SEARCH_PARAMS.CELL}=${item.id}`
    );
  };

  const handleCreateCanvas = (value: string) => {
    setIsCommandMenuOpen(false);
    setCanvasPageState(CanvasesPageState.CREATE_CANVAS_DIALOG);
    navigate(APP_ROUTES.CANVASES);
    return value;
  };

  const handleCreateConversation = (value: string) => {
    setIsCommandMenuOpen(false);
    navigate(APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.INDEX);
    return value;
  };

  useKeyboardShortcut(
    () => handleCreateConversation(""),
    KEYBOARD_SHORTCUTS.NEW_CONVERSATION
  );

  const renderContent = () => {
    if (inputValue !== "" || !isHomePage) {
      return (
        <CommandMenuSearchResults
          data={searchQuery.data}
          heading={inputValue === "" ? "Recent" : "Search Results"}
          onItemSelect={handleSearchResultSelect}
        />
      );
    }
    return (
      <React.Fragment>
        <CommandMenuHomePage
          onSelect={handlePageSelect}
          onCreateCanvas={handleCreateCanvas}
          onCreateConversation={handleCreateConversation}
        />
        <CommandMenuSearchResults
          data={searchQuery.data?.slice(0, RECENT_ITEMS_LIMIT)}
          heading="Recent"
          onItemSelect={handleSearchResultSelect}
        />
      </React.Fragment>
    );
  };

  return (
    <Command.Dialog
      open={isCommandMenuOpen}
      onOpenChange={setIsCommandMenuOpen}
      label="Global Command Menu"
      onKeyDown={handleKeyDown}
      filter={filter}
    >
      <Flex gap="xs">
        {pages.map((p) => (
          <Badge key={p} variant="subtle">
            {PageToDisplayNameMap[p]}
          </Badge>
        ))}
      </Flex>
      <Command.Input
        autoFocus
        placeholder="What do you need ?"
        onValueChange={(value: string) => {
          setInputValue(value);
        }}
      />
      <Command.List>
        <Command.Empty>No results found</Command.Empty>
        {renderContent()}
      </Command.List>
    </Command.Dialog>
  );
};
