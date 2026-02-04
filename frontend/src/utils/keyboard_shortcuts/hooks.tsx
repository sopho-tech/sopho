import { RefObject, useEffect, useRef, useLayoutEffect } from "react";
import {
  handleKeyboardShortcut,
  KeyboardShortcut,
} from "src/utils/keyboard_shortcuts/keyboard_shortcuts";

export function useKeyboardShortcut(
  callback: (event: KeyboardEvent) => void,
  keymap: KeyboardShortcut,
  htmlElementToAttachToRef?: RefObject<HTMLDivElement | null>
) {
  const handleCallBack = handleKeyboardShortcut(keymap, callback);

  const handleCallBackRef = useRef(handleCallBack);

  useLayoutEffect(() => {
    handleCallBackRef.current = handleCallBack;
  });

  useEffect(() => {
    if (!htmlElementToAttachToRef) {
      document.addEventListener("keydown", handleCallBack);
      return () => {
        document.removeEventListener("keydown", handleCallBack);
      };
    }

    const element = htmlElementToAttachToRef.current;
    if (element != null) {
      element.addEventListener("keydown", handleCallBack);
      return () => {
        element.removeEventListener("keydown", handleCallBack);
      };
    }
  }, [handleCallBack, htmlElementToAttachToRef]);
}
