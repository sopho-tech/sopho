import { StateCreator } from "zustand";

export type CommandMenuSlice = {
  commandMenu: {
    isOpen: boolean;
    setIsOpen: (isOpen: boolean) => void;
    toggleIsOpen: () => void;
  };
};

export const createCommandMenuSlice: StateCreator<CommandMenuSlice> = (set) => ({
  commandMenu: {
    isOpen: false,
    setIsOpen: (isOpen) =>
      set((state) => ({ commandMenu: { ...state.commandMenu, isOpen } })),
    toggleIsOpen: () =>
      set((state) => ({
        commandMenu: { ...state.commandMenu, isOpen: !state.commandMenu.isOpen },
      })),
  },
});
