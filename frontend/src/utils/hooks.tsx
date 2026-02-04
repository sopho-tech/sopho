import { useState, useEffect, useMemo } from "react";
import debounce from "lodash.debounce";

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
