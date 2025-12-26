import { Table } from "@tanstack/react-table";
import styles from "src/components/design-system/DataTable/DataTable.module.css";

export function createDragPreview<T>(
  rowElement: HTMLTableRowElement,
  table: Table<T>
): HTMLDivElement {
  const tableWidth = table.getCenterTotalSize();

  const dragPreviewContainer = document.createElement("div");
  dragPreviewContainer.className = styles.dragPreviewContainer;

  const dragPreview = document.createElement("table");
  dragPreview.className = styles.dragPreview;
  dragPreview.style.width = `${tableWidth}px`;
  dragPreview.style.borderCollapse = "collapse";
  dragPreview.style.tableLayout = "fixed";

  const previewTbody = document.createElement("tbody");
  previewTbody.className = styles.dragPreviewRow;

  const previewRow = rowElement.cloneNode(true) as HTMLTableRowElement;
  previewRow.className = `${styles.tableBodyRow} ${styles.dragPreviewRow}`;

  const cells = previewRow.querySelectorAll("td");
  cells.forEach((cell) => {
    cell.className = `${cell.className} ${styles.dragPreviewCell}`;
  });

  previewTbody.appendChild(previewRow);
  dragPreview.appendChild(previewTbody);
  dragPreviewContainer.appendChild(dragPreview);
  document.body.appendChild(dragPreviewContainer);
  dragPreviewContainer.style.position = "absolute";
  dragPreviewContainer.style.top = "-9999px";
  dragPreviewContainer.style.left = "-9999px";

  return dragPreviewContainer;
}

