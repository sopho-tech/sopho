export type SpacingSize = "2xs" | "xs" | "sm" | "md" | "lg" | "xl" | "2xl";

export type ButtonSize = "sm" | "md" | "lg";

export type IconButtonSize = "none" | "sm" | "md" | "lg";

export type ButtonType = "submit" | "button";

export type ButtonEmphasis = "primary" | "secondary" | "tertiary";

export type BorderRadius = "sm" | "md" | "lg" | "xl" | "2xl" | "full";

export type IconColor =
  | "accent"
  | "white"
  | "black"
  | "default"
  | "info"
  | "error"
  | "warning"
  | "green"
  | "grey"
  | "lightgrey"
  | "red"
  | "transparent";

export type IconType =
  | "add"
  | "home"
  | "book"
  | "settings"
  | "close"
  | "circle_x"
  | "swap_horiz"
  | "swap_vert"
  | "visibility"
  | "visibility_off"
  | "edit"
  | "delete"
  | "clear"
  | "play"
  | "remove"
  | "arrow_up"
  | "arrow_down"
  | "more_horiz"
  | "more_vert"
  | "chevron_left"
  | "chevron_right"
  | "chevron_down"
  | "chevron_up"
  | "check"
  | "triangle_alert"
  | "hash"
  | "type"
  | "calendar"
  | "check_square"
  | "square"
  | "square_check"
  | "key"
  | "info"
  | "grip_vertical"
  | "star"
  | "link"
  | "save"
  | "circle_x"
  | "search"
  | "sparkles"
  | "refresh"
  | "email"
  | "lock"
  | "user"
  | "logout"
  | "message"
  | "layers"
  | "bar_chart"
  | "table"
  | "layout_dashboard"
  | "plug"
  | "bot"
  | "panel"
  | "copy"
  | "database"
  | "scan_search"
  | "brain"
  | "telescope"
  | "workflow"
  | "circle_check"
  | "circle_dot"
  | "file_text"
  | "wand_sparkles";

export type IconSize = "sm" | "md" | "lg" | "2xl";

export type IconAnimation = "pulse";

export type FlexValue = "grow" | "shrink" | "none" | NumberString;

export type OverflowValue = "scrollY" | "scrollX" | "hidden";

export type Direction = "row" | "column" | "row-reverse" | "column-reverse";

export type JustifyContent =
  | "flex-start"
  | "flex-end"
  | "center"
  | "space-between"
  | "space-around"
  | "space-evenly";

export type AlignItems =
  | "flex-start"
  | "flex-end"
  | "center"
  | "stretch"
  | "baseline";

export type AlignContent =
  | "flex-start"
  | "flex-end"
  | "center"
  | "space-between"
  | "space-around"
  | "space-evenly"
  | "stretch";

export type ColorVariant =
  | "default"
  | "info"
  | "error"
  | "warning"
  | "white"
  | "grey";

export type BorderVariant = "default" | "divider" | "warning";

export type ShadowVariant = "2xs" | "xs" | "sm" | "md" | "lg" | "xl" | "scroll";

export type Postition = "sticky" | "absolute" | "relative" | "fixed";

export type ZIndex = "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10";

export type Display = "flex";

export type NumberString = number | string;

export type SpacingValue = SpacingSize | `-${SpacingSize}` | NumberString;

export type BoxElement =
  | "div"
  | "article"
  | "aside"
  | "caption"
  | "details"
  | "figcaption"
  | "figure"
  | "footer"
  | "header"
  | "main"
  | "nav"
  | "p"
  | "section"
  | "summary"
  | "ul"
  | "li";

export type SharedLayoutProps = {
  borderRadius?: BorderRadius;
  border?: BorderVariant;
  borderTop?: BorderVariant;
  borderRight?: BorderVariant;
  borderBottom?: BorderVariant;
  borderLeft?: BorderVariant;
  shadow?: ShadowVariant;
  backgroundColor?: ColorVariant;
  direction?: Direction;
  gap?: SpacingSize;
  paddingX?: SpacingSize;
  paddingY?: SpacingSize;
  paddingTop?: SpacingSize;
  paddingRight?: SpacingSize;
  paddingBottom?: SpacingSize;
  paddingLeft?: SpacingSize;
  marginTop?: SpacingValue;
  marginBottom?: SpacingValue;
  marginLeft?: SpacingValue;
  marginRight?: SpacingValue;
  flex?: FlexValue;
  overflow?: OverflowValue;
  justifyContent?: JustifyContent;
  alignItems?: AlignItems;
  alignContent?: AlignContent;
  alignSelf?: AlignItems;
  top?: SpacingValue;
  bottom?: SpacingValue;
  left?: SpacingValue;
  right?: SpacingValue;
  position?: Postition;
  zIndex?: ZIndex;
  height?: NumberString;
  width?: NumberString;
  ref?: React.Ref<HTMLElement>;
};
