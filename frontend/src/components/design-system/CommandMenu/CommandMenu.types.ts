import { SearchResultItemDto } from "src/api/search/queries";

export type Page = "home" | "canvases" | "sql_cells" | "chart_cells";

export type CommandMenuHomePageProps = {
  onSelect: (page: Page) => void;
  onCreateCanvas: (value: string) => void;
};

export type CommandMenuSearchResultsProps = {
  data: SearchResultItemDto[] | undefined;
  onItemSelect: (item: SearchResultItemDto) => void;
};

