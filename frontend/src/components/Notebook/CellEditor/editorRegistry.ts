import { EditorView } from "@codemirror/view";

const editorViews = new Map<string, EditorView>();

export function registerEditorView(cellId: string, view: EditorView) {
  editorViews.set(cellId, view);
}

export function unregisterEditorView(cellId: string, view: EditorView) {
  if (editorViews.get(cellId) === view) {
    editorViews.delete(cellId);
  }
}

export function getSelectedText(cellId: string): string | null {
  const view = editorViews.get(cellId);
  if (!view) return null;

  const { from, to } = view.state.selection.main;
  if (from === to) return null;

  const selectedText = view.state.sliceDoc(from, to).trim();
  return selectedText.length > 0 ? selectedText : null;
}
