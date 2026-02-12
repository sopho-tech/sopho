import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { useCanvas, useUpdateCanvas } from "src/api/canvas/queries";
import { useNotebooksByCanvasId } from "src/api/notebook/queries";
import { Flex, InlineEdit } from "src/components/design-system";
import { Notebook } from "src/components/Notebook";
import { useStore } from "src/store";
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
  const updateCanvas = useUpdateCanvas();
  const [viewType, setViewType] = useState<ViewType>(ViewType.NOTEBOOK);
  const setActiveNotebookId = useStore(
    (state) => state.canvas.setActiveNotebookId
  );

  useEffect(() => {
    const firstNotebookId =
      notebooksQuery.data &&
      notebooksQuery.data.length > 0 &&
      notebooksQuery.data[0].id
        ? notebooksQuery.data[0].id
        : "";
    setActiveNotebookId(firstNotebookId);
  }, [notebooksQuery.data, setActiveNotebookId]);

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
          <InlineEdit
            value={query.data.name ?? ""}
            onSave={(name) =>
              updateCanvas.mutate({
                canvasId: params.id!,
                payload: { ...query.data, name: name },
              })
            }
            headingLevel={1}
            clearButtonSize="md"
            defaultValue="Default Canvas Title"
          />
          <CanvasButtons
            viewType={viewType.toLowerCase()}
            onViewTypeChange={(v: string) =>
              setViewType(ViewType[v.toUpperCase() as keyof typeof ViewType])
            }
          />
        </Flex>
        <InlineEdit
          value={query.data.description ?? ""}
          onSave={(description) =>
            updateCanvas.mutate({
              canvasId: params.id!,
              payload: { ...query.data, description },
            })
          }
          placeholder="Add a description"
          defaultValue="Default description"
          textColor="subtle"
        />
      </Flex>
      {renderView()}
    </Flex>
  );
}
