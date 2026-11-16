import { StateCreator } from "zustand";

const LAST_TENANT_STORAGE_KEY = "sopho.tenant.last";

function readLastTenantId(): string | null {
  if (typeof window === "undefined") {
    return null;
  }
  return window.localStorage.getItem(LAST_TENANT_STORAGE_KEY);
}

export type TenantSlice = {
  tenant: {
    lastTenantId: string | null;
    setLastTenantId: (lastTenantId: string) => void;
  };
};

export const createTenantSlice: StateCreator<TenantSlice> = (set) => ({
  tenant: {
    lastTenantId: readLastTenantId(),
    setLastTenantId: (lastTenantId) => {
      window.localStorage.setItem(LAST_TENANT_STORAGE_KEY, lastTenantId);
      set((state) => ({ tenant: { ...state.tenant, lastTenantId } }));
    },
  },
});
