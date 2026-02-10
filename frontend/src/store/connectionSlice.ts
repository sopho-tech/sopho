import { StateCreator } from "zustand";
import {
  ConnectionDetailsPageStateEnum,
  ConnectionDto,
} from "src/components/Connection/dto";

export type ConnectionSlice = {
  connection: {
    connectionDetailsPageState: ConnectionDetailsPageStateEnum;
    connectionId: string;
    connections: ConnectionDto[];
    setConnectionDetailsPageState: (
      pageState: ConnectionDetailsPageStateEnum,
    ) => void;
    setConnectionId: (id: string) => void;
    setConnections: (connections: ConnectionDto[]) => void;
  };
};

export const createConnectionSlice: StateCreator<ConnectionSlice> = (set) => ({
  connection: {
    connections: [],
    connectionDetailsPageState: ConnectionDetailsPageStateEnum.LIST,
    connectionId: "",
    setConnectionDetailsPageState: (pageState) =>
      set((state) => ({
        connection: { ...state.connection, connectionDetailsPageState: pageState },
      })),
    setConnectionId: (id) =>
      set((state) => ({ connection: { ...state.connection, connectionId: id } })),
    setConnections: (connections) =>
      set((state) => ({ connection: { ...state.connection, connections } })),
  },
});
