import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { ExecutionState } from "src/components/Notebook/Cell/dto";
import { useStore } from "src/store";

const initialStoreState = useStore.getState();
const CELL_ID = "5f3b5c3e-4b8e-4f43-9a51-9a2d6a4c1b01";

describe("cell slice", () => {
  beforeEach(() => {
    useStore.setState(initialStoreState, true);
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-15T10:00:00Z"));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  test("records when a cell starts running", () => {
    useStore.getState().cell.setExecutionState(CELL_ID, ExecutionState.RUNNING);

    const cell = useStore.getState().cell;
    expect(cell.executionStates[CELL_ID]).toBe(ExecutionState.RUNNING);
    expect(cell.startedAt[CELL_ID]).toBe(Date.parse("2026-10-15T10:00:00Z"));
  });

  test("forgets the start time once the cell finishes", () => {
    const cell = useStore.getState().cell;
    cell.setExecutionState(CELL_ID, ExecutionState.RUNNING);
    cell.setExecutionState(CELL_ID, ExecutionState.COMPLETED);

    const finishedCell = useStore.getState().cell;
    expect(finishedCell.executionStates[CELL_ID]).toBe(
      ExecutionState.COMPLETED,
    );
    expect(finishedCell.startedAt[CELL_ID]).toBeUndefined();
  });

  test("clears both the state and the start time", () => {
    const cell = useStore.getState().cell;
    cell.setExecutionState(CELL_ID, ExecutionState.RUNNING);
    cell.clearExecutionState(CELL_ID);

    const clearedCell = useStore.getState().cell;
    expect(clearedCell.executionStates[CELL_ID]).toBeUndefined();
    expect(clearedCell.startedAt[CELL_ID]).toBeUndefined();
  });
});
