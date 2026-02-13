import { useEffect, useRef } from "react";
import debounce from "lodash.debounce";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import {
  EditorView,
  keymap,
  lineNumbers,
  highlightSpecialChars,
  drawSelection,
  dropCursor,
  rectangularSelection,
  crosshairCursor,
  highlightActiveLineGutter,
  highlightActiveLine,
} from "@codemirror/view";
import { EditorState, Prec } from "@codemirror/state";
import {
  bracketMatching,
  foldGutter,
  foldKeymap,
  indentOnInput,
} from "@codemirror/language";
import { sql, PostgreSQL } from "@codemirror/lang-sql";
import {
  autocompletion,
  closeBrackets,
  closeBracketsKeymap,
  completionKeymap,
} from "@codemirror/autocomplete";
import { useCell, useUpdateCell, cellKeys } from "src/api/cell/queries";
import { useQueryClient } from "@tanstack/react-query";
import { CellDto } from "src/components/Notebook/Cell/dto";
import CellEditorStyles from "src/components/Notebook/CellEditor/CellEditor.module.css";
import { NOTEBOOK_CELL_KEYBOARD_SHORTCUTS } from "src/utils/keyboard_shortcuts";
import { KeyBinding } from "@codemirror/view";
import { lintKeymap } from "@codemirror/lint";
import { theme } from "./theme";

export function CellEditor({ cellId }: { cellId: string }) {
  const query = useCell(cellId);
  const queryClient = useQueryClient();
  const editorRef = useRef<HTMLDivElement>(null);
  const updateCellMutation = useUpdateCell();

  useEffect(() => {
    let debouncedUpdate: ReturnType<typeof debounce> | undefined;

    if (editorRef.current && query.data) {
      const customKeymap: KeyBinding[] = NOTEBOOK_CELL_KEYBOARD_SHORTCUTS.map(
        (shortcut) => {
          const modifiers = shortcut.modifiers || [];
          let keyBinding: string = shortcut.key;

          const hasModifier = (mod: string) => modifiers.some((m) => m === mod);

          if (hasModifier("meta")) {
            keyBinding = `Mod-${keyBinding}`;
          }
          if (hasModifier("shift")) {
            keyBinding = `Shift-${keyBinding}`;
          }
          if (hasModifier("alt")) {
            keyBinding = `Alt-${keyBinding}`;
          }
          if (hasModifier("ctrl")) {
            keyBinding = `Ctrl-${keyBinding}`;
          }

          return {
            key: keyBinding,
            run: () => {
              return true;
            },
          };
        }
      );

      debouncedUpdate = debounce((content: string) => {
        const cell = queryClient.getQueryData<CellDto>(cellKeys.detail(cellId));
        if (cell) {
          updateCellMutation.mutate({
            cellId: cellId,
            payload: { ...cell, content },
          });
        }
      }, 300);

      let state = EditorState.create({
        doc: query.data.content || "",
        extensions: [
          theme,
          lineNumbers(),
          highlightSpecialChars(),
          history(),
          foldGutter(),
          drawSelection(),
          dropCursor(),
          indentOnInput(),
          bracketMatching(),
          closeBrackets(),
          rectangularSelection(),
          crosshairCursor(),
          highlightActiveLine(),
          highlightActiveLineGutter(),
          autocompletion(),
          sql({
            dialect: PostgreSQL,
          }),
          Prec.highest(keymap.of(customKeymap)),
          keymap.of([
            ...closeBracketsKeymap,
            ...defaultKeymap,
            ...historyKeymap,
            ...foldKeymap,
            ...completionKeymap,
            ...lintKeymap,
          ]),
          EditorState.tabSize.of(2),
          EditorState.allowMultipleSelections.of(false),
          EditorView.updateListener.of((update) => {
            if (!update.docChanged) return;
            debouncedUpdate?.(update.state.doc.toJSON().join("\n"));
          }),
          EditorView.lineWrapping,
        ],
      });
      new EditorView({
        parent: editorRef.current,
        state: state,
      });
    }

    return () => {
      debouncedUpdate?.cancel();
    };
  }, [cellId, query.isPending]);

  return <div ref={editorRef} className={CellEditorStyles.container} />;
}
