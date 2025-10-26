import { RefObject, useEffect, useRef, useLayoutEffect } from "react";
import {
  handleKeyboardShortcut,
  KeyboardShortcut,
} from "src/utils/keyboard_shortcuts/keyboard_shortcuts";

export function useKeyboardShortcut(
  htmlElementToAttachToRef: RefObject<HTMLDivElement | null>,
  callback: (event: KeyboardEvent) => void,
  keymap: KeyboardShortcut
) {
  const handleCallBack = handleKeyboardShortcut(keymap, callback);

  const handleCallBackRef = useRef(handleCallBack);

  useLayoutEffect(() => {
    handleCallBackRef.current = handleCallBack;
  });

  useEffect(() => {
    const element = htmlElementToAttachToRef.current;
    if (element != null) {
      element.addEventListener("keydown", handleCallBack);
      return () => {
        element.removeEventListener("keydown", handleCallBack);
      };
    }
  }, [handleCallBack, htmlElementToAttachToRef]);
}
