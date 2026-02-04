import { Flex, MotionFlex, Heading } from "src/components/design-system";
import { AccessbilityLevel } from "../Heading/Heading";
import { Scale, Duration, EASE } from "../animation";

type CardProps = {
  className?: string;
  children: React.ReactNode;
  onClick?: () => void;
};

const CardRoot = ({ className, children, onClick }: CardProps) => {
  const baseProps = {
    backgroundColor: "default" as const,
    border: "default" as const,
    borderRadius: "lg" as const,
    paddingX: "xl" as const,
    paddingY: "md" as const,
    direction: "column" as const,
    gap: "lg" as const,
    className: className,
  };

  if (onClick) {
    return (
      <MotionFlex
        {...baseProps}
        onClick={onClick}
        whileHover={{ scale: Scale.HOVER }}
        whileTap={{ scale: Scale.TAP }}
        transition={{
          duration: Duration.FAST,
          ease: EASE,
        }}
        sx={{ cursor: "pointer" }}
      >
        {children}
      </MotionFlex>
    );
  }

  return <Flex {...baseProps}>{children}</Flex>;
};

CardRoot.displayName = "Card";

type CardHeaderProps = {
  children: React.ReactNode;
};

const CardHeader = ({ children }: CardHeaderProps) => {
  return (
    <Flex direction="column" gap="sm">
      {children}
    </Flex>
  );
};

CardHeader.displayName = "Card.Header";

type CardTitleProps = {
  accessbilityLevel: AccessbilityLevel;
  children: React.ReactNode;
};

const CardTitle = ({ accessbilityLevel, children }: CardTitleProps) => {
  return (
    <Heading accessbilityLevel={accessbilityLevel} size="lg" weight="medium">
      {children}
    </Heading>
  );
};

CardTitle.displayName = "Card.Title";

type CardSubtitleProps = {
  accessbilityLevel?: AccessbilityLevel;
  children: React.ReactNode;
};

const CardSubtitle = ({ accessbilityLevel, children }: CardSubtitleProps) => {
  return (
    <Heading
      accessbilityLevel={accessbilityLevel || 3}
      size="base"
      weight="normal"
      textColor="subtle"
    >
      {children}
    </Heading>
  );
};

CardSubtitle.displayName = "Card.Subtitle";

type CardContentProps = {
  children: React.ReactNode;
};

const CardContent = ({ children }: CardContentProps) => {
  return <>{children}</>;
};

CardContent.displayName = "Card.Content";

export const Card = Object.assign(CardRoot, {
  Header: CardHeader,
  Title: CardTitle,
  Subtitle: CardSubtitle,
  Content: CardContent,
});
