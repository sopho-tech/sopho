import { Duration, EASE, Scale } from "src/components/design-system/animation";
import { Flex } from "src/components/design-system/Flex";
import {
  AccessibilityLevel,
  Heading,
} from "src/components/design-system/Heading/Heading";
import { MotionFlex } from "src/components/design-system/MotionFlex";
import { useContext, createContext } from "react";

type CardVariant = "outline" | "ghost" | "filled";

type CardRootProps = {
  children: React.ReactNode;
  className?: string;
  variant?: CardVariant;
  onClick?: () => void;
  sx?: React.CSSProperties;
};

type CardContextType = {
  variant?: CardVariant;
};

const CardContext = createContext<CardContextType | null>(null);

const CardRoot = ({
  children,
  className,
  variant,
  onClick,
  sx,
}: CardRootProps) => {
  const contextValue = {
    variant: variant,
  };
  return (
    <CardContext value={contextValue}>
      <MotionFlex
        onClick={onClick}
        whileHover={{ scale: Scale.HOVER }}
        whileTap={{ scale: Scale.TAP }}
        transition={{
          duration: Duration.FAST,
          ease: EASE,
        }}
        sx={{ cursor: "pointer", ...sx }}
        className={className}
        backgroundColor="default"
        border="default"
        borderRadius="lg"
        paddingX="xl"
        paddingY="md"
        direction="column"
        gap="lg"
        height="100%"
        justifyContent="space-between"
      >
        {children}
      </MotionFlex>
    </CardContext>
  );
};

CardRoot.displayName = "Card";

type CardHeaderProps = {
  children: React.ReactNode;
  baseAccessibilityLevel: AccessibilityLevel;
};

type CardHeaderContextType = {
  baseAccessibilityLevel: AccessibilityLevel;
};

const CardHeaderContext = createContext<CardHeaderContextType | null>(null);

const useCardHeaderContext = () => {
  const context = useContext(CardHeaderContext);
  if (!context) {
    throw "CardHeader compound components must be used within CardHeader parent component";
  }
  return context;
};

const CardHeader = ({ children, baseAccessibilityLevel }: CardHeaderProps) => {
  const contextValue = {
    baseAccessibilityLevel: baseAccessibilityLevel,
  };
  return (
    <CardHeaderContext value={contextValue}>
      <Flex direction="column" gap="sm">
        {children}
      </Flex>
    </CardHeaderContext>
  );
};

CardHeader.displayName = "Card.Header";

type CardTitleProps = {
  children: string | null;
};

const CardTitle = ({ children }: CardTitleProps) => {
  const { baseAccessibilityLevel } = useCardHeaderContext();
  return (
    <Heading
      accessibilityLevel={baseAccessibilityLevel}
      size="lg"
      weight="medium"
    >
      {children}
    </Heading>
  );
};

CardTitle.displayName = "Card.Title";

type CardDescriptionProps = CardTitleProps;

const CardDescription = ({ children }: CardDescriptionProps) => {
  const { baseAccessibilityLevel } = useCardHeaderContext();
  const accessibilityLevel = (baseAccessibilityLevel + 1) as AccessibilityLevel;
  return (
    <Heading
      accessibilityLevel={accessibilityLevel}
      size="base"
      weight="normal"
      textColor="subtle"
    >
      {children}
    </Heading>
  );
};

CardDescription.displayName = "Card.Description";

type CardContentProps = {
  children: React.ReactNode;
};

const CardContent = ({ children }: CardContentProps) => {
  return (
    <Flex direction="column" gap="sm">
      {children}
    </Flex>
  );
};

CardContent.displayName = "Card.Content";

export const Card = Object.assign(CardRoot, {
  Header: CardHeader,
  Title: CardTitle,
  Description: CardDescription,
  Content: CardContent,
});
