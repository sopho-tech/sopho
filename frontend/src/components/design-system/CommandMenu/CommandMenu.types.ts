import { SearchResultItemDto } from "src/api/search/queries";

export type Page =
  | "home"
  | "canvases"
  | "sql_cells"
  | "chart_cells"
  | "conversations";

export type CommandMenuHomePageProps = {
  onSelect: (page: Page) => void;
  onCreateCanvas: (value: string) => void;
  onCreateConversation: (value: string) => void;
};

export type CommandMenuSearchResultsProps = {
  data: SearchResultItemDto[] | undefined;
  heading: string;
  onItemSelect: (item: SearchResultItemDto) => void;
};
