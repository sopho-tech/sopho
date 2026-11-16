import { UsageSortColumn } from "src/api/ai_usage";

export type UsageListHeader = {
  label: string;
  sortColumn: UsageSortColumn;
  align: "start" | "end";
};

export type UsageListPage = {
  total_pages: number;
  page: number;
  page_size: number;
  total_items: number;
};
