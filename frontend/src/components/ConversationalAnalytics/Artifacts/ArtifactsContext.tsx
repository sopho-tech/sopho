import { createContext, useCallback, useContext, useState } from "react";
import { useConversationArtifacts } from "src/components/ConversationalAnalytics/Artifacts/useConversationArtifacts";
import type { Artifact } from "src/components/ConversationalAnalytics/dto";

type ArtifactsContextValue = {
  artifacts: Artifact[];
  isPanelOpen: boolean;
  togglePanel: () => void;
  closePanel: () => void;
};

const ArtifactsContext = createContext<ArtifactsContextValue | null>(null);

type ArtifactsProviderProps = {
  conversationId: string;
  children: React.ReactNode;
};

export const ArtifactsProvider = ({
  conversationId,
  children,
}: ArtifactsProviderProps) => {
  const artifacts = useConversationArtifacts(conversationId);
  const [isOpen, setIsOpen] = useState(false);

  const togglePanel = useCallback(() => setIsOpen((open) => !open), []);
  const closePanel = useCallback(() => setIsOpen(false), []);

  const value: ArtifactsContextValue = {
    artifacts,
    isPanelOpen: isOpen && artifacts.length > 0,
    togglePanel,
    closePanel,
  };

  return <ArtifactsContext value={value}>{children}</ArtifactsContext>;
};

export const useArtifacts = (): ArtifactsContextValue => {
  const context = useContext(ArtifactsContext);
  if (!context) {
    throw new Error("useArtifacts must be used within ArtifactsProvider");
  }
  return context;
};
