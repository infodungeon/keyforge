import { useRef, useEffect, useCallback } from "react";

/**
 * A hook that provides a ref to automatically focus an element when it mounts,
 * and a manual trigger to re-focus it (e.g. after a state change).
 */
export function useAutoFocus<T extends HTMLInputElement | HTMLTextAreaElement>(
  delay = 50,
) {
  const ref = useRef<T>(null);

  const triggerFocus = useCallback(() => {
    // We use a small timeout because sometimes React hasn't finished
    // updating the DOM or clearing the previous value/state
    setTimeout(() => {
      if (ref.current) {
        ref.current.focus();
      }
    }, delay);
  }, [delay]);

  useEffect(() => {
    triggerFocus();
  }, [triggerFocus]);

  return { ref, triggerFocus };
}
