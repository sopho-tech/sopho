import { useCallback, useEffect, useState } from "react";
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
  const params = useParams();
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

  const handleNameSave = useCallback(
    (name: string) =>
      updateCanvas.mutate({
        canvasId: params.id!,
        payload: { ...query.data!, name },
      }),
    [params.id, query.data, updateCanvas]
  );

  const handleDescriptionSave = useCallback(
    (description: string) =>
      updateCanvas.mutate({
        canvasId: params.id!,
        payload: { ...query.data!, description },
      }),
    [params.id, query.data, updateCanvas]
  );

  const handleViewTypeChange = useCallback((v: string) => {
    setViewType(ViewType[v.toUpperCase() as keyof typeof ViewType]);
  }, []);

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
      paddingX="2xl"
      paddingY="md"
      gap="xl"
      flex="grow"
      overflow="scrollY"
    >
      <Flex direction="column" gap="sm">
        <Flex direction="row" justifyContent="space-between">
          <InlineEdit
            value={query.data.name ?? ""}
            onSave={handleNameSave}
            headingLevel={1}
            clearButtonSize="md"
            defaultValue="Default Canvas Title"
          />
          <CanvasButtons
            viewType={viewType.toLowerCase()}
            onViewTypeChange={handleViewTypeChange}
          />
        </Flex>
        <InlineEdit
          value={query.data.description ?? ""}
          onSave={handleDescriptionSave}
          placeholder="Add a description"
          defaultValue="Default description"
          textColor="subtle"
        />
      </Flex>
      {renderView()}
    </Flex>
  );
}
