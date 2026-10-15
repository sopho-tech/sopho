export default {
  extends: ["stylelint-config-standard", "stylelint-config-css-modules"],
  ignoreFiles: [
    "**/node_modules/**",
    "**/dist/**",
    "**/.next/**",
    "**/coverage/**",
  ],
  rules: {
    "selector-class-pattern": [
      "^(?:[a-z][a-zA-Z0-9]*|react-(?:draggable|grid|resizable)(?:-[a-z]+)*|is-editor-empty)$",
      {
        message:
          "Expected class selector to be camelCase (third-party react-grid-layout and tiptap classes excepted)",
      },
    ],
    "keyframes-name-pattern": [
      "^[a-z][a-zA-Z0-9]*$",
      { message: "Expected keyframe name to be camelCase" },
    ],
    "at-rule-no-unknown": [true, { ignoreAtRules: ["theme", "value"] }],
    "import-notation": "string",
    "property-no-vendor-prefix": [
      true,
      { ignoreProperties: ["-webkit-user-select"] },
    ],
    "block-no-empty": [true, { severity: "warning" }],
    "no-descending-specificity": [true, { severity: "warning" }],
  },
};
