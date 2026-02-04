import {
  MotionBox,
  type MotionBoxProps,
} from "src/components/design-system/MotionBox/MotionBox";

export type MotionFlexProps = Omit<MotionBoxProps, "display">;

export function MotionFlex(props: MotionFlexProps) {
  return <MotionBox display="flex" {...props} />;
}
