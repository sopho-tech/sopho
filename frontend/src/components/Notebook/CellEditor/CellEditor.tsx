import { useEffect, useRef } from "react";
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
} from "@codemirror/view";
import { EditorState, Prec } from "@codemirror/state";
import {
  bracketMatching,
  foldGutter,
  foldKeymap,
  indentOnInput,
  syntaxHighlighting,
  HighlightStyle,
} from "@codemirror/language";
import { sql, PostgreSQL } from "@codemirror/lang-sql";
import { autocompletion, completionKeymap } from "@codemirror/autocomplete";
import { tags } from "@lezer/highlight";
import { useCell, useUpdateCell } from "src/api/cell/queries";
import CellEditorStyles from "src/components/Notebook/CellEditor/CellEditor.module.css";
import "src/components/Notebook/CellEditor/CellEditor.global.css";
import { getCSSVariable } from "src/utils/css_util";
import { useStore } from "src/store";
import { StateEffect } from "@codemirror/state";
import { NOTEBOOK_CELL_KEYBOARD_SHORTCUTS } from "src/utils/keyboard_shortcuts";
import { KeyBinding } from "@codemirror/view";

const myHighlightStyle = HighlightStyle.define([
  { tag: tags.keyword, color: getCSSVariable("--color-primary-600") },
  {
    tag: tags.comment,
    color: getCSSVariable("--color-grey"),
    fontStyle: "italic",
  },
  { tag: tags.string, color: getCSSVariable("--color-red-dark-2") },
  { tag: tags.number, color: getCSSVariable("--color-foreground") },
  { tag: tags.operator, color: getCSSVariable("--color-foreground") },
  { tag: tags.punctuation, color: getCSSVariable("--color-foreground") },
  { tag: tags.variableName, color: getCSSVariable("--color-foreground") },
  { tag: tags.typeName, color: getCSSVariable("--color-primary-600") },
]);

export function CellEditor({ cellId }: { cellId: string }) {
  const query = useCell(cellId);
  const editorRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const updateCellMutation = useUpdateCell();
  const setActiveCellId = useStore((state) => state.notebook.setActiveCellId);
  const smokeEffect = StateEffect.define(undefined);

  useEffect(() => {
    if (editorRef.current && !viewRef.current && query.data) {
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

      let state = EditorState.create({
        doc: query.data.content || "",
        extensions: [
          lineNumbers(),
          highlightSpecialChars(),
          history(),
          foldGutter(),
          drawSelection(),
          dropCursor(),
          indentOnInput(),
          bracketMatching(),
          rectangularSelection(),
          crosshairCursor(),
          highlightActiveLineGutter(),
          syntaxHighlighting(myHighlightStyle, { fallback: true }),
          autocompletion(),
          sql({
            dialect: PostgreSQL,
          }),
          Prec.highest(keymap.of(customKeymap)),
          keymap.of([
            ...defaultKeymap,
            ...historyKeymap,
            ...foldKeymap,
            ...completionKeymap,
          ]),
          EditorState.tabSize.of(2),
          EditorState.allowMultipleSelections.of(true),
          EditorView.updateListener.of((update) => {
            if (query.data) {
              updateCellMutation.mutate({
                cellId: cellId,
                payload: {
                  ...query.data,
                  content: update.state.doc.toJSON().join("\n"),
                },
              });
            }
          }),
          EditorView.focusChangeEffect.of((_, focusing) => {
            if (focusing) {
              setActiveCellId(cellId);
            }
            return smokeEffect.of(null);
          }),
        ],
      });
      const view = new EditorView({
        parent: editorRef.current,
        state: state,
      });
      viewRef.current = view;
    }

    return () => {
      if (viewRef.current) {
        viewRef.current.destroy();
        viewRef.current = null;
      }
    };
  }, [cellId, query.isPending]);

  return <div ref={editorRef} className={CellEditorStyles.container} />;
}
