import PaginationStyles from "src/components/Pagination/Pagination.module.css";
import { Select } from "src/components/Select";
import { Button } from "src/components/design-system/Button";

export type PaginationProps = {
  totalPages: number;
  currentPage: number;
  pageSize: number;
  totalItems: number;
  onChangePageSize: (newValue: string) => void;
  onPageClick: (newPage: number) => void;
  containerClassName?: string;
};

type PageItem = number | "ellipsis-left" | "ellipsis-right";

function getPageItems(currentPage: number, totalPages: number): PageItem[] {
  if (totalPages <= 7) {
    return Array.from({ length: totalPages }, (_, i) => i);
  }

  const items: PageItem[] = [];
  const isNearStart = currentPage <= 3;
  const isNearEnd = currentPage >= totalPages - 4;

  if (isNearStart) {
    for (let i = 0; i <= 4; i++) items.push(i);
    items.push("ellipsis-right");
    items.push(totalPages - 1);
  } else if (isNearEnd) {
    items.push(0);
    items.push("ellipsis-left");
    for (let i = totalPages - 5; i < totalPages; i++) items.push(i);
  } else {
    items.push(0);
    items.push("ellipsis-left");
    for (let i = currentPage - 1; i <= currentPage + 1; i++) items.push(i);
    items.push("ellipsis-right");
    items.push(totalPages - 1);
  }

  return items;
}

function renderPageButton(
  pageIndex: number,
  currentPage: number,
  onPageClick: (newPage: number) => void
): React.ReactElement {
  return (
    <Button
      key={pageIndex}
      label={(pageIndex + 1).toString()}
      shape="square"
      backgroundColor="white"
      size="sm"
      disabled={pageIndex === currentPage}
      onClick={({ event }) => onPageClick(pageIndex)}
    />
  );
}

function renderEllipsisButton(
  direction: "left" | "right",
  currentPage: number,
  totalPages: number,
  jumpAmount: number,
  onPageClick: (newPage: number) => void
): React.ReactElement {
  const newPage =
    direction === "left"
      ? Math.max(0, currentPage - jumpAmount)
      : Math.min(totalPages - 1, currentPage + jumpAmount);
  return (
    <Button
      key={`ellipsis-${direction}`}
      label=""
      shape="square"
      backgroundColor="white"
      size="sm"
      leadingIconName="more_horiz"
      onClick={({ event }) => onPageClick(newPage)}
    />
  );
}

function renderPages(
  currentPage: number,
  totalPages: number,
  onPageClick: (newPage: number) => void
): React.ReactElement[] {
  const jumpAmount = 5;
  const pageItems = getPageItems(currentPage, totalPages);

  return [
    <Button
      key="first"
      label=""
      shape="square"
      backgroundColor="white"
      size="sm"
      leadingIconName="chevron_left"
      disabled={currentPage === 0}
      onClick={({ event }) => onPageClick(Math.max(0, currentPage - 1))}
    />,
    ...pageItems.map((item) =>
      typeof item === "number"
        ? renderPageButton(item, currentPage, onPageClick)
        : renderEllipsisButton(
            item === "ellipsis-left" ? "left" : "right",
            currentPage,
            totalPages,
            jumpAmount,
            onPageClick
          )
    ),
    <Button
      key="last"
      label=""
      shape="square"
      backgroundColor="white"
      size="sm"
      leadingIconName="chevron_right"
      disabled={currentPage === totalPages - 1}
      onClick={({ event }) =>
        onPageClick(Math.min(totalPages - 1, currentPage + 1))
      }
    />,
  ];
}

export function Pagination({
  totalPages,
  currentPage,
  pageSize,
  totalItems,
  onChangePageSize,
  onPageClick,
  containerClassName,
}: PaginationProps) {
  const pages = renderPages(currentPage, totalPages, onPageClick);

  function handleValueChange(newValue: string) {
    onChangePageSize(newValue);
  }

  return (
    <div
      className={`${PaginationStyles.container} ${containerClassName || ""}`}
    >
      <p className={PaginationStyles.summary}>
        {pageSize * currentPage + 1} -{" "}
        {Math.min(totalItems, pageSize * (currentPage + 1))} of {totalItems}{" "}
        items{" "}
      </p>
      <div className={PaginationStyles.pageButtonsContainer}>{pages}</div>
      <div className={PaginationStyles.pageSizeSelectionContainer}>
        <span className={PaginationStyles.pageSizeSelectionLabel}>
          Rows per page:
        </span>
        <Select
          placeholderText="Select page size"
          groupName="Page Sizes"
          value={pageSize.toString()}
          onValueChange={handleValueChange}
          options={[
            { label: "10", value: "10" },
            { label: "20", value: "20" },
            { label: "50", value: "50" },
            { label: "100", value: "100" },
          ]}
        />
      </div>
    </div>
  );
}
