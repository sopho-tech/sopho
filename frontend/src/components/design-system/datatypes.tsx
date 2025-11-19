export type SpacingSize = "2xs" | "xs" | "sm" | "md" | "lg" | "xl" | "2xl";

export type ButtonSize = "sm" | "md" | "lg";

export type ButtonType = "submit" | "button";

export type ButtonEmphasis = "primary" | "secondary" | "tertiary";

export type IconColor =
  | "accent"
  | "white"
  | "black"
  | "default"
  | "info"
  | "error"
  | "warning";

export type IconType =
  | "add"
  | "home"
  | "book_2"
  | "settings"
  | "close"
  | "swap_horiz"
  | "swap_vert";

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

export type ColorVariant = "default" | "info" | "error" | "warning";

export type Postition = "sticky" | "absolute" | "relative";

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
  | "summary";

export type SharedLayoutProps = {
  direction?: Direction;
  gap?: SpacingSize;
  paddingX?: SpacingSize;
  paddingY?: SpacingSize;
  marginTop?: SpacingValue;
  marginBottom?: SpacingValue;
  marginLeft?: SpacingValue;
  marginRight?: SpacingValue;
  flex?: FlexValue;
  overflow?: OverflowValue;
  justifyContent?: JustifyContent;
  alignItems?: AlignItems;
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
