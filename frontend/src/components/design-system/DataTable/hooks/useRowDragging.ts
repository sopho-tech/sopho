import { useState, useRef } from "react";
import { Table, Row, RowData } from "@tanstack/react-table";
import type { DataTableFeatures } from "src/components/design-system/DataTable/features";
import { createDragPreview } from "src/components/design-system/DataTable/components/DragPreview";

export function useRowDragging<T extends RowData>(
  table: Table<DataTableFeatures, T>,
) {
  const [draggingRowId, setDraggingRowId] = useState<string | null>(null);
  const dragPreviewRef = useRef<HTMLDivElement | null>(null);

  const handleDragStart = (
    e: React.DragEvent<HTMLTableRowElement>,
    row: Row<DataTableFeatures, T>,
  ) => {
    setDraggingRowId(row.id);
    const rowElement = e.currentTarget;

    const dragPreviewContainer = createDragPreview(rowElement, table);

    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setDragImage(dragPreviewContainer, 0, 0);
    e.dataTransfer.setData("application/x-sopho-cell-id", row.id);

    dragPreviewRef.current = dragPreviewContainer;

    setTimeout(() => {
      if (dragPreviewRef.current) {
        document.body.removeChild(dragPreviewRef.current);
        dragPreviewRef.current = null;
      }
    }, 0);
  };

  const handleDragEnd = () => {
    setDraggingRowId(null);
    if (dragPreviewRef.current) {
      document.body.removeChild(dragPreviewRef.current);
      dragPreviewRef.current = null;
    }
  };

  return {
    draggingRowId,
    handleDragStart,
    handleDragEnd,
  };
}
