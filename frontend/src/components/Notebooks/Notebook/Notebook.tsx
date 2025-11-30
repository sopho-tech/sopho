import { useParams } from "react-router";
import { Cell } from "src/components/Notebooks/Notebook/Cell";
import { ChartCell } from "src/components/Notebooks/Notebook/ChartCell";
import { NotebookMenuBar } from "src/components/Notebooks/Notebook/NotebookMenuBar";
import { useNotebookStore } from "src/components/Notebooks/store";
import { useEffect, useRef, useState } from "react";
import { useNotebook } from "src/api/notebook/queries";
import { CellType } from "src/components/Notebooks/Notebook/Cell/dto";
import { KEYBOARD_SHORTCUTS } from "src/utils/keyboard_shortcuts";
import { useHandleExecuteCell } from "src/components/Notebooks/Notebook/Cell";
import { useKeyboardShortcut } from "src/utils/keyboard_shortcuts/hooks";
import { Heading, SegmentedControl } from "src/components/design-system";
import { Text } from "src/components/design-system/Text/Text";
import { Flex } from "src/components/design-system/Flex/Flex";
import { Sticky } from "src/components/design-system/Sticky/Sticky";

enum ViewType {
  NOTEBOOK = "NOTEBOOK",
  DASHBOARD = "DASHBOARD",
}

export function Notebook() {
  let params = useParams();
  const { currentNotebookId, setCurrentNotebookId, activeCellId } =
    useNotebookStore();
  const query = useNotebook(params.id!);
  const handleExecuteCell = useHandleExecuteCell();
  const notebookRef = useRef<HTMLDivElement>(null);
  const [viewType, setViewType] = useState<ViewType>(ViewType.NOTEBOOK);

  function handleExecute() {
    handleExecuteCell(activeCellId);
  }

  useKeyboardShortcut(
    notebookRef,
    handleExecute,
    KEYBOARD_SHORTCUTS.EXECUTE_NOTEBOOK_CELL
  );

  useEffect(() => {
    if (currentNotebookId !== params.id) {
      setCurrentNotebookId(params.id!);
    }
  }, [params.id]);

  if (query.isPending) {
    return <span>Loading...</span>;
  }

  if (query.isError) {
    return <span>Error...{query.error.message}</span>;
  }

  if (!query.data) {
    return <span>No data available</span>;
  }

  function generateCellComponents() {
    if (!query.data?.cells) return null;
    return query.data.cells
      .sort((a, b) => {
        const orderA = a.display_order ?? Number.MAX_SAFE_INTEGER;
        const orderB = b.display_order ?? Number.MAX_SAFE_INTEGER;
        return orderA - orderB;
      })
      .map((cell) => {
        if (cell.cell_type === CellType.CHART) {
          return <ChartCell key={cell.id} cell_id={cell.id} />;
        }
        return <Cell key={cell.id} cell_id={cell.id} />;
      });
  }

  return (
    <Flex
      ref={notebookRef}
      direction="column"
      paddingX="md"
      paddingY="md"
      gap="md"
      flex="grow"
      overflow="scrollY"
    >
      <Flex direction="column" gap="sm">
        <Flex direction="row" justifyContent="space-between">
          <Heading accessbilityLevel={1}>{query.data.name}</Heading>
          <SegmentedControl
            options={[
              {
                label: "Notebook",
                value: "notebook",
                leadingIcon: "book",
              },
              {
                label: "Dashboard",
                value: "dashboard",
                leadingIcon: "calendar",
              },
            ]}
            value={viewType.toLowerCase()}
            onValueChange={(v) =>
              setViewType(ViewType[v.toUpperCase() as keyof typeof ViewType])
            }
            size="md"
          />
        </Flex>
        <Text as="p" color="subtle">
          {query.data.description}
        </Text>
      </Flex>
      <Sticky top={0} zIndex={"10"}>
        <NotebookMenuBar />
      </Sticky>
      <Flex direction="column" gap="md">
        {generateCellComponents()}
      </Flex>
    </Flex>
  );
}
