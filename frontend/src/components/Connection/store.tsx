import { ConnectionDetailsPageStateEnum } from "./dto";
import { create } from "zustand";
import { ConnectionDto } from "src/components/Connection/dto";

interface ConnectionsStore {
  connectionDetailsPageState: ConnectionDetailsPageStateEnum;
  connectionId: string;
  connections: ConnectionDto[];
  setConnectionDetailsPageState: (
    pageState: ConnectionDetailsPageStateEnum,
  ) => void;
  setConnectionId: (id: string) => void;
  setConnections: (connections: ConnectionDto[]) => void;
}

export const useConnectionsStore = create<ConnectionsStore>((set) => ({
  connections: [],
  connectionDetailsPageState: ConnectionDetailsPageStateEnum.LIST,
  setConnectionDetailsPageState: (pageState) =>
    set(() => ({ connectionDetailsPageState: pageState })),
  connectionId: "",
  setConnectionId: (id) => set(() => ({ connectionId: id })),
  setConnections: (connections) => set(() => ({ connections: connections })),
}));
