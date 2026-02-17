import { EditorView } from "@codemirror/view";
import { getCSSVariable } from "src/utils/css_util";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";
import { Extension } from "@codemirror/state";

const textColor = getCSSVariable("--color-grey-900");
const editorBackground = getCSSVariable("--color-grey-100");
const cursorColor = getCSSVariable("--color-primary-500");
const selectionBackground = getCSSVariable("--color-primary-100");
const panelBackground = getCSSVariable("--color-grey-200");
const gutterColor = getCSSVariable("--color-grey-600");
const activeLineBackground = getCSSVariable("--color-grey-200");
const tooltipBackground = getCSSVariable("--color-grey-100");
const fontFamily = getCSSVariable("--font-family-mono");
const spaceMd = getCSSVariable("--space-sm");
const space2xs = getCSSVariable("--space-2xs");

const keywordColor = getCSSVariable("--color-primary-600");
const nameColor = getCSSVariable("--color-grey-800");
const functionColor = getCSSVariable("--color-primary-500");
const constantColor = getCSSVariable("--color-grey-700");
const definitionColor = getCSSVariable("--color-grey-900");
const typeColor = getCSSVariable("--color-grey-700");
const operatorColor = getCSSVariable("--color-grey-600");
const commentColor = getCSSVariable("--color-grey-500");
const stringColor = getCSSVariable("--color-green-600");
const invalidColor = getCSSVariable("--color-red-500");

const editorTheme = EditorView.theme(
  {
    "&": {
      color: textColor,
      backgroundColor: editorBackground,
    },

    "&.cm-focused": {
      outline: "none",
    },

    ".cm-content": {
      caretColor: cursorColor,
      fontFamily: fontFamily,
      paddingTop: spaceMd,
      paddingBottom: spaceMd,
    },

    ".cm-lineNumbers .cm-gutterElement": {
      textAlign: "center",
      alignContent: "center",
    },

    ".cm-line": {
      paddingTop: space2xs,
      paddingBottom: space2xs,
    },

    ".cm-cursor, .cm-dropCursor": { borderLeftColor: cursorColor },
    "&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection":
      { backgroundColor: selectionBackground },

    ".cm-panels": { backgroundColor: panelBackground, color: textColor },
    ".cm-panels.cm-panels-top": { borderBottom: "2px solid black" },
    ".cm-panels.cm-panels-bottom": { borderTop: "2px solid black" },

    ".cm-searchMatch": {
      backgroundColor: editorBackground,
      outline: "1px solid #457dff",
    },
    ".cm-searchMatch.cm-searchMatch-selected": {
      backgroundColor: "#6199ff2f",
    },

    ".cm-activeLine": { backgroundColor: "#6699ff0b" },
    ".cm-selectionMatch": { backgroundColor: "#aafe661a" },

    "&.cm-focused .cm-matchingBracket, &.cm-focused .cm-nonmatchingBracket": {
      backgroundColor: "#bad0f847",
    },

    ".cm-gutters": {
      backgroundColor: editorBackground,
      color: gutterColor,
      border: "none",
    },

    ".cm-activeLineGutter": {
      backgroundColor: activeLineBackground,
    },

    ".cm-foldPlaceholder": {
      backgroundColor: "transparent",
      border: "none",
      color: "#ddd",
    },

    ".cm-tooltip": {
      border: "none",
      backgroundColor: tooltipBackground,
    },
    ".cm-tooltip .cm-tooltip-arrow:before": {
      borderTopColor: "transparent",
      borderBottomColor: "transparent",
    },
    ".cm-tooltip .cm-tooltip-arrow:after": {
      borderTopColor: tooltipBackground,
      borderBottomColor: tooltipBackground,
    },
    ".cm-tooltip-autocomplete": {
      color: textColor,
      border: `1px solid ${getCSSVariable("--color-grey-300")}`,
      "& > ul > li": {
        color: textColor,
      },
      "& > ul > li[aria-selected]": {
        backgroundColor: activeLineBackground,
        color: textColor,
      },
    },
  },
  { dark: true }
);

export const highlightStyle = HighlightStyle.define([
  { tag: t.keyword, color: keywordColor },
  {
    tag: [t.name, t.deleted, t.character, t.propertyName, t.macroName],
    color: nameColor,
  },
  { tag: [t.function(t.variableName), t.labelName], color: functionColor },
  {
    tag: [t.color, t.constant(t.name), t.standard(t.name)],
    color: constantColor,
  },
  { tag: [t.definition(t.name), t.separator], color: definitionColor },
  {
    tag: [
      t.typeName,
      t.className,
      t.number,
      t.changed,
      t.annotation,
      t.modifier,
      t.self,
      t.namespace,
    ],
    color: typeColor,
  },
  {
    tag: [
      t.operator,
      t.operatorKeyword,
      t.url,
      t.escape,
      t.regexp,
      t.link,
      t.special(t.string),
    ],
    color: operatorColor,
  },
  { tag: [t.meta, t.comment], color: commentColor },
  { tag: t.strong, fontWeight: "bold" },
  { tag: t.emphasis, fontStyle: "italic" },
  { tag: t.strikethrough, textDecoration: "line-through" },
  { tag: t.link, color: commentColor, textDecoration: "underline" },
  { tag: t.heading, fontWeight: "bold", color: nameColor },
  { tag: [t.atom, t.bool, t.special(t.variableName)], color: constantColor },
  { tag: [t.processingInstruction, t.string, t.inserted], color: stringColor },
  { tag: t.invalid, color: invalidColor },
]);

export const theme: Extension = [
  editorTheme,
  syntaxHighlighting(highlightStyle),
];
