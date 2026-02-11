import PaginationStyles from "src/components/design-system/Pagination/Pagination.module.css";
import { Select } from "src/components/design-system";
import { Button } from "src/components/design-system/Button";
import { IconButton } from "src/components/design-system/IconButton/IconButton";

export type PaginationProps = {
  totalPages: number;
  currentPage: number;
  pageSize: number;
  totalItems: number;
  onChangePageSize: (newValue: string) => void;
  onPageClick: (newPage: number) => void;
  containerClassName?: string;
  showRowsPerPage?: boolean;
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
  const isActive = pageIndex === currentPage;
  return (
    <Button
      key={pageIndex}
      label={(pageIndex + 1).toString()}
      shape="square"
      backgroundColor={isActive ? "lightgrey" : "white"}
      size="sm"
      disabled={isActive}
      onClick={() => onPageClick(pageIndex)}
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
      onClick={() => onPageClick(newPage)}
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
    <IconButton
      key="first"
      type="chevron_left"
      backgroundColor="white"
      iconColor="grey"
      iconSize="sm"
      onClick={() => {
        if (currentPage > 0) {
          onPageClick(Math.max(0, currentPage - 1));
        }
      }}
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
    <IconButton
      key="last"
      type="chevron_right"
      backgroundColor="white"
      iconColor="grey"
      iconSize="sm"
      onClick={() => {
        if (currentPage < totalPages - 1) {
          onPageClick(Math.min(totalPages - 1, currentPage + 1));
        }
      }}
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
  showRowsPerPage = true,
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
      {showRowsPerPage && (
        <div className={PaginationStyles.pageSizeSelectionContainer}>
          <span className={PaginationStyles.pageSizeSelectionLabel}>
            Rows per page:
          </span>
          <Select value={pageSize.toString()} onValueChange={handleValueChange}>
            <Select.Trigger placeholder="Select page size" />
            <Select.Content>
              <Select.Group>
                <Select.Label>Page Sizes</Select.Label>
                <Select.Item value="10">10</Select.Item>
                <Select.Item value="20">20</Select.Item>
                <Select.Item value="50">50</Select.Item>
                <Select.Item value="100">100</Select.Item>
              </Select.Group>
            </Select.Content>
          </Select>
        </div>
      )}
    </div>
  );
}
