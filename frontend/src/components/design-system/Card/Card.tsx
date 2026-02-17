import { Duration, EASE, Scale } from "src/components/design-system/animation";
import { Flex } from "src/components/design-system/Flex";
import {
  AccessbilityLevel,
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
};

type CardContextType = {
  variant?: CardVariant;
};

const CardContext = createContext<CardContextType | null>(null);

const CardRoot = ({ children, className, variant, onClick }: CardRootProps) => {
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
        sx={{ cursor: "pointer" }}
        className={className}
        backgroundColor="default"
        border="default"
        borderRadius="lg"
        paddingX="xl"
        paddingY="md"
        direction="column"
        gap="lg"
      >
        {children}
      </MotionFlex>
    </CardContext>
  );
};

CardRoot.displayName = "Card";

type CardHeaderProps = {
  children: React.ReactNode;
  baseAccessbilityLevel: AccessbilityLevel;
};

type CardHeaderContextType = {
  baseAccessbilityLevel: AccessbilityLevel;
};

const CardHeaderContext = createContext<CardHeaderContextType | null>(null);

const useCardHeaderContext = () => {
  const context = useContext(CardHeaderContext);
  if (!context) {
    throw "CardHeader compound components must be used within CardHeader parent component";
  }
  return context;
};

const CardHeader = ({ children, baseAccessbilityLevel }: CardHeaderProps) => {
  const contextValue = {
    baseAccessbilityLevel: baseAccessbilityLevel,
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
  const { baseAccessbilityLevel } = useCardHeaderContext();
  return (
    <Heading
      accessbilityLevel={baseAccessbilityLevel}
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
  const { baseAccessbilityLevel } = useCardHeaderContext();
  const accessbilityLevel = (baseAccessbilityLevel + 1) as AccessbilityLevel;
  return (
    <Heading
      accessbilityLevel={accessbilityLevel}
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
