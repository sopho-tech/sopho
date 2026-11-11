import { StateCreator } from "zustand";

const SIDEBAR_PINNED_STORAGE_KEY = "sopho.sidebar.pinned";

function readIsPinnedPreference(): boolean {
  if (typeof window === "undefined") {
    return false;
  }
  return window.localStorage.getItem(SIDEBAR_PINNED_STORAGE_KEY) === "true";
}

export type SidebarSlice = {
  sidebar: {
    isPinned: boolean;
    setIsPinned: (isPinned: boolean) => void;
  };
};

export const createSidebarSlice: StateCreator<SidebarSlice> = (set) => ({
  sidebar: {
    isPinned: readIsPinnedPreference(),
    setIsPinned: (isPinned) => {
      window.localStorage.setItem(SIDEBAR_PINNED_STORAGE_KEY, String(isPinned));
      set((state) => ({ sidebar: { ...state.sidebar, isPinned } }));
    },
  },
});
