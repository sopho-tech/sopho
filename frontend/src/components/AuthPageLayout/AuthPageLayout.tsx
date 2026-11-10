import { useMemo } from "react";
import { MeshGradient } from "@paper-design/shaders-react";
import { useReducedMotion } from "motion/react";
import { Flex } from "src/components/design-system";
import styles from "src/components/AuthPageLayout/AuthPageLayout.module.css";
import {
  AUTH_PAGE_GRADIENT_COLORS,
  AUTH_PAGE_GRADIENT_SPEED,
  AUTH_PAGE_GRADIENT_STILL_SPEED,
} from "src/components/AuthPageLayout/constants";
import logo from "src/assets/images/logo.svg";

type AuthPageLayoutProps = {
  children: React.ReactNode;
};

export function AuthPageLayout({ children }: AuthPageLayoutProps) {
  const meshGradientColors = useMemo(() => [...AUTH_PAGE_GRADIENT_COLORS], []);
  const prefersReducedMotion = useReducedMotion();

  return (
    <Flex
      width={"100vw"}
      height={"100vh"}
      justifyContent="center"
      alignItems="center"
      className={styles.pageContainer}
    >
      <MeshGradient
        colors={meshGradientColors}
        distortion={0.8}
        swirl={0.1}
        grainMixer={0.47}
        grainOverlay={0.3}
        speed={
          prefersReducedMotion
            ? AUTH_PAGE_GRADIENT_STILL_SPEED
            : AUTH_PAGE_GRADIENT_SPEED
        }
        className={styles.backgroundGradient}
      />
      <Flex
        direction="column"
        shadow="sm"
        borderRadius="lg"
        paddingX="xl"
        paddingY="xl"
        gap="sm"
        className={styles.formContainer}
        backgroundColor="white"
      >
        <Flex justifyContent="center" alignItems="center" marginBottom="md">
          <img src={logo} alt="Logo" className={styles.logo} />
        </Flex>
        {children}
      </Flex>
    </Flex>
  );
}
