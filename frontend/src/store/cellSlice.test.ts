import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { ExecutionState } from "src/components/Notebook/Cell/dto";
import { useStore } from "src/store";
import { SQL_CELL } from "src/test/test_data";

const initialStoreState = useStore.getState();

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
    useStore
      .getState()
      .cell.setExecutionState(SQL_CELL.id, ExecutionState.RUNNING);

    const cell = useStore.getState().cell;
    expect(cell.executionStates[SQL_CELL.id]).toBe(ExecutionState.RUNNING);
    expect(cell.startedAt[SQL_CELL.id]).toBe(
      Date.parse("2026-10-15T10:00:00Z"),
    );
  });

  test("forgets the start time once the cell finishes", () => {
    const cell = useStore.getState().cell;
    cell.setExecutionState(SQL_CELL.id, ExecutionState.RUNNING);
    cell.setExecutionState(SQL_CELL.id, ExecutionState.COMPLETED);

    const finishedCell = useStore.getState().cell;
    expect(finishedCell.executionStates[SQL_CELL.id]).toBe(
      ExecutionState.COMPLETED,
    );
    expect(finishedCell.startedAt[SQL_CELL.id]).toBeUndefined();
  });

  test("clears both the state and the start time", () => {
    const cell = useStore.getState().cell;
    cell.setExecutionState(SQL_CELL.id, ExecutionState.RUNNING);
    cell.clearExecutionState(SQL_CELL.id);

    const clearedCell = useStore.getState().cell;
    expect(clearedCell.executionStates[SQL_CELL.id]).toBeUndefined();
    expect(clearedCell.startedAt[SQL_CELL.id]).toBeUndefined();
  });
});
