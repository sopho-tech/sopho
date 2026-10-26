import type {
  GrantAccessLevel,
  ResourceVisibility,
} from "src/api/grant/queries";

export const VISIBILITY_OPTIONS: {
  value: ResourceVisibility;
  label: string;
}[] = [
  { value: "PRIVATE", label: "Only people with access" },
  { value: "TENANT", label: "Everyone in this tenant" },
];

export const ACCESS_LEVEL_OPTIONS: {
  value: GrantAccessLevel;
  label: string;
}[] = [
  { value: "VIEWER", label: "Can view" },
  { value: "EDITOR", label: "Can edit" },
];

export const DEFAULT_ACCESS_LEVEL: GrantAccessLevel = "VIEWER";

export const REMOVE_ACCESS_TOOLTIP = { text: "Remove access" };
