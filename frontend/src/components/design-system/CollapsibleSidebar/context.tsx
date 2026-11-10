import { createContext, useContext } from "react";

type CollapsibleSidebarContextValue = {
  isExpanded: boolean;
  isPinned: boolean;
  setIsPinned: (isPinned: boolean) => void;
  setIsHeldOpen: (isHeldOpen: boolean) => void;
};

export const CollapsibleSidebarContext =
  createContext<CollapsibleSidebarContextValue | null>(null);

export function useCollapsibleSidebar() {
  const context = useContext(CollapsibleSidebarContext);
  if (!context) {
    throw new Error(
      "useCollapsibleSidebar must be used within a CollapsibleSidebar",
    );
  }
  return context;
}
