const React = window.React;

if (!React) {
  throw new Error("React not found on window. Ensure react UMD is loaded.");
}

export default React;
export const {
  useState,
  useEffect,
  useMemo,
  useCallback,
  useRef,
  useContext,
  createContext,
  createElement,
  Fragment,
  memo,
  forwardRef,
  useLayoutEffect,
  useReducer,
  useId,
  useSyncExternalStore,
} = React;
