import { useState, useEffect, useMemo } from "react";
import debounce from "lodash.debounce";

const ELEMENT_LOOKUP_TIMEOUT_MS = 3000;

export function useScrollToElement(
  elementId: string | null,
  onScrolled?: () => void
) {
  useEffect(() => {
    if (!elementId) {
      return;
    }

    let frameId = 0;
    const startedAt = performance.now();

    const scrollWhenAvailable = () => {
      const element = document.getElementById(elementId);
      if (element) {
        element.scrollIntoView({ behavior: "smooth", block: "center" });
        onScrolled?.();
        return;
      }
      if (performance.now() - startedAt > ELEMENT_LOOKUP_TIMEOUT_MS) {
        return;
      }
      frameId = requestAnimationFrame(scrollWhenAvailable);
    };

    frameId = requestAnimationFrame(scrollWhenAvailable);
    return () => cancelAnimationFrame(frameId);
  }, [elementId, onScrolled]);
}

export function useDebouncedValue<T>(value: T, delay: number = 300): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  const debouncedSetValue = useMemo(
    () => debounce((val: T) => setDebouncedValue(val), delay),
    [delay]
  );

  useEffect(() => {
    debouncedSetValue(value);
    return () => debouncedSetValue.cancel();
  }, [value, debouncedSetValue]);

  return debouncedValue;
}
