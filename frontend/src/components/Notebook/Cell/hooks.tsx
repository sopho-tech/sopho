import { useExecuteCell, useExecuteCellPreview } from "src/api/cell";

export function useHandleExecuteCell() {
  const executeCellMutation = useExecuteCell();

  return (
    cellId: string,
    onSuccessCallback: (() => void) | null = null,
    onErrorCallback: (() => void) | null = null
  ) => {
    executeCellMutation.mutate(cellId, {
      onSuccess: () => {
        if (onSuccessCallback) {
          onSuccessCallback();
        }
      },
      onError: (error) => {
        if (onErrorCallback) {
          onErrorCallback();
        }
      },
    });
  };
}

export function useHandleExecuteCellPreview() {
  const executeCellPreviewMutation = useExecuteCellPreview();

  return (
    cellId: string,
    content: string,
    cellType: string,
    onSuccessCallback?: () => void,
    onErrorCallback?: () => void
  ) => {
    executeCellPreviewMutation.mutate(
      { cellId, content, cellType },
      {
        onSuccess: () => {
          onSuccessCallback?.();
        },
        onError: () => {
          onErrorCallback?.();
        },
      }
    );
  };
}
