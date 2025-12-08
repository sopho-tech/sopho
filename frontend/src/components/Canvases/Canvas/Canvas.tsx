import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { useCanvas } from "src/api/canvas/queries";
import { useNotebooksByCanvasId } from "src/api/notebook/queries";
import { Flex, Heading, Text } from "src/components/design-system";
import { Notebook } from "src/components/Notebook";
import { useCanvasStore } from "src/components/Canvases/store";
import { Dashboard } from "src/components/Dashboard";
import { CanvasButtons } from "src/components/Canvases/CanvasButtons";

enum ViewType {
  NOTEBOOK = "NOTEBOOK",
  DASHBOARD = "DASHBOARD",
}

export function Canvas() {
  let params = useParams();
  const query = useCanvas(params.id!);
  const notebooksQuery = useNotebooksByCanvasId(params.id!);
  const [viewType, setViewType] = useState<ViewType>(ViewType.NOTEBOOK);
  const { setActiveNotebookId } = useCanvasStore();

  useEffect(() => {
    const firstNotebookId =
      notebooksQuery.data &&
      notebooksQuery.data.length > 0 &&
      notebooksQuery.data[0].id
        ? notebooksQuery.data[0].id
        : "";
    setActiveNotebookId(firstNotebookId);
  }, [notebooksQuery.data]);

  if (!query.data) {
    return <span>No data available</span>;
  }

  const renderView = () => {
    if (viewType == ViewType.DASHBOARD) {
      return <Dashboard />;
    }
    return <Notebook />;
  };

  return (
    <Flex
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
          <CanvasButtons
            viewType={viewType.toLowerCase()}
            onViewTypeChange={(v: string) =>
              setViewType(ViewType[v.toUpperCase() as keyof typeof ViewType])
            }
          />
        </Flex>
        <Text as="p" color="subtle">
          {query.data.description}
        </Text>
      </Flex>
      {renderView()}
    </Flex>
  );
}
