import { useEffect, useState } from "react";
import { ExecutionState } from "src/components/Notebook/Cell/dto";
import { useStore } from "src/store";

export enum ExecutionPhase {
  IDLE = "IDLE",
  SETTLING = "SETTLING",
  VISIBLE = "VISIBLE",
  EXTENDED = "EXTENDED",
  LONG = "LONG",
}

const SETTLE_MS = 300;
const EXTENDED_MS = 2000;
const LONG_MS = 5000;

const PHASE_STEPS: Array<[number, ExecutionPhase]> = [
  [SETTLE_MS, ExecutionPhase.VISIBLE],
  [EXTENDED_MS, ExecutionPhase.EXTENDED],
  [LONG_MS, ExecutionPhase.LONG],
];

export function useExecutionPhase(cellId: string) {
  const executionState = useStore(
    (state) => state.cell.executionStates[cellId],
  );
  const startedAt = useStore((state) => state.cell.startedAt[cellId]);
  const [phase, setPhase] = useState(ExecutionPhase.IDLE);

  const isRunning = executionState === ExecutionState.RUNNING;

  useEffect(() => {
    if (!isRunning || !startedAt) {
      setPhase(ExecutionPhase.IDLE);
      return;
    }

    setPhase(ExecutionPhase.SETTLING);
    const timers = PHASE_STEPS.map(([threshold, nextPhase]) =>
      setTimeout(
        () => setPhase(nextPhase),
        threshold - (Date.now() - startedAt),
      ),
    );

    return () => timers.forEach(clearTimeout);
  }, [isRunning, startedAt]);

  return { isRunning, phase, startedAt };
}

export function isAtLeast(phase: ExecutionPhase, target: ExecutionPhase) {
  const order = [
    ExecutionPhase.IDLE,
    ExecutionPhase.SETTLING,
    ExecutionPhase.VISIBLE,
    ExecutionPhase.EXTENDED,
    ExecutionPhase.LONG,
  ];
  return order.indexOf(phase) >= order.indexOf(target);
}
