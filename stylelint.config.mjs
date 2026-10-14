export default {
  extends: ["stylelint-config-standard", "stylelint-config-css-modules"],
  rules: {
    "selector-class-pattern": [
      "^[a-z][a-zA-Z0-9]*$",
      { message: "Expected class selector to be camelCase" },
    ],
    "keyframes-name-pattern": [
      "^[a-z][a-zA-Z0-9]*$",
      { message: "Expected keyframe name to be camelCase" },
    ],
    "at-rule-no-unknown": [true, { ignoreAtRules: ["theme"] }],
  },
};
