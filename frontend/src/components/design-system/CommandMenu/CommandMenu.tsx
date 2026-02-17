// TODO Incorporate fix of https://github.com/dip/cmdk/issues/393 when released

import { Command } from "cmdk";
import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router";
import { Badge } from "src/components/design-system";
import { Flex } from "src/components/design-system";
import { SearchResultItemDto, useSearch } from "src/api/search/queries";
import { EntityType } from "src/api/search";
import { APP_ROUTES } from "src/constants/app_routes";
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

type CommandMenuProps = {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
};

const PageToDisplayNameMap: Record<Page, string> = {
  home: "Home",
  canvases: "Canvases",
  sql_cells: "SQL Cells",
  chart_cells: "Chart Cells",
};

const getFilters = (currentPage: Page) => {
  if (currentPage === "home") {
    return [EntityType.Canvas, EntityType.ChartCell, EntityType.SqlCell];
  }
  if (currentPage === "canvases") {
    return [EntityType.Canvas];
  }
  if (currentPage === "sql_cells") {
    return [EntityType.SqlCell];
  }
  if (currentPage === "chart_cells") {
    return [EntityType.ChartCell];
  }
  return [];
};

export const CommandMenu = ({
  open: controlledOpen,
  onOpenChange,
}: CommandMenuProps) => {
  const navigate = useNavigate();
  const setCanvasPageState = useStore((state) => state.canvas.setCanvasPageState);
  const [internalOpen, setInternalOpen] = useState(false);
  const isControlled = controlledOpen !== undefined;
  const open = isControlled ? controlledOpen : internalOpen;
  const [inputValue, setInputValue] = useState<string>("");
  const debouncedInputValue = useDebouncedValue(inputValue);
  const [pages, setPages] = useState<Page[]>(["home"]);
  const currentPage = pages[pages.length - 1];
  const isHomePage = currentPage === "home";
  const filters = getFilters(currentPage);
  const searchQuery = useSearch(debouncedInputValue, filters);

  const handleOpenChange = (newOpen: boolean) => {
    if (isControlled) {
      onOpenChange?.(newOpen);
    } else {
      setInternalOpen(newOpen);
    }
  };

  const commandMenuCallback = () => {
    if (isControlled) {
      onOpenChange?.(!open);
    } else {
      setInternalOpen((prevOpen) => !prevOpen);
    }
  };

  useKeyboardShortcut(
    commandMenuCallback,
    KEYBOARD_SHORTCUTS.OPEN_COMMAND_MENU
  );

  useEffect(() => {
    if (open) {
      setPages(["home"]);
      setInputValue("");
    }
  }, [open]);

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
      handleOpenChange(false);
      navigate(APP_ROUTES.CANVAS.replace(":id", item.id));
    }
  };

  const handleCreateCanvas = (value: string) => {
    handleOpenChange(false);
    setCanvasPageState(CanvasesPageState.CREATE_CANVAS_DIALOG);
    navigate(APP_ROUTES.CANVASES);
    return value;
  };

  const renderContent = () => {
    if (inputValue === "" && !isHomePage) {
      return (
        <CommandMenuSearchResults
          data={searchQuery.data}
          onItemSelect={handleSearchResultSelect}
        />
      );
    }
    if (inputValue !== "") {
      return (
        <CommandMenuSearchResults
          data={searchQuery.data}
          onItemSelect={handleSearchResultSelect}
        />
      );
    }
    if (isHomePage && inputValue === "") {
      return (
        <CommandMenuHomePage
          onSelect={handlePageSelect}
          onCreateCanvas={handleCreateCanvas}
        />
      );
    }
    return null;
  };

  return (
    <Command.Dialog
      open={open}
      onOpenChange={handleOpenChange}
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
