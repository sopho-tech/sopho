import { Flex } from "src/components/design-system/Flex/Flex";
import { Children, isValidElement } from "react";

const FLEX_BORDER_STYLE = { borderBottom: "var(--border-divider)" };
const FLEX_MAX_WIDTH_STYLE = { maxWidth: "50%" };

type TopBarSectionProps = {
  children?: React.ReactNode;
};

function TopBarLeft({ children }: TopBarSectionProps) {
  return <>{children}</>;
}

function TopBarCenter({ children }: TopBarSectionProps) {
  return <>{children}</>;
}

function TopBarRight({ children }: TopBarSectionProps) {
  return <>{children}</>;
}

type TopBarProps = {
  children?: React.ReactNode;
  className?: string;
};

function TopBarRoot({ children, className }: TopBarProps) {
  let leftContent: React.ReactNode = null;
  let centerContent: React.ReactNode = null;
  let rightContent: React.ReactNode = null;

  Children.forEach(children, (child) => {
    if (isValidElement(child) && "props" in child) {
      const props = child.props as { children?: React.ReactNode };
      if (child.type === TopBarLeft) {
        leftContent = props.children;
      } else if (child.type === TopBarCenter) {
        centerContent = props.children;
      } else if (child.type === TopBarRight) {
        rightContent = props.children;
      }
    }
  });

  return (
    <Flex
      direction="row"
      justifyContent="space-between"
      alignItems="center"
      paddingX="lg"
      paddingY="xs"
      sx={FLEX_BORDER_STYLE}
      backgroundColor="white"
      className={className}
    >
      <Flex>{leftContent}</Flex>
      <Flex justifyContent="center" sx={FLEX_MAX_WIDTH_STYLE}>
        {centerContent}
      </Flex>
      <Flex>{rightContent}</Flex>
    </Flex>
  );
}

export const TopBar = Object.assign(TopBarRoot, {
  Left: TopBarLeft,
  Center: TopBarCenter,
  Right: TopBarRight,
});
