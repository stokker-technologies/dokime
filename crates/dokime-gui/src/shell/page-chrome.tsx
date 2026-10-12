/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";

/* === Types === */

export type PageChrome = {
  /** Title Case page title rendered at the left inset of the page bar. */
  title: string;
  /** The single action slot at the right inset. Memoize it; see `usePageChrome`. */
  actions: ReactNode;
};

type PageChromeInput = {
  title: string;
  actions?: ReactNode;
};

/* === Constants === */

const EMPTY_CHROME: PageChrome = { actions: null, title: "" };

const PageChromeStateContext = createContext<PageChrome>(EMPTY_CHROME);
const PageChromePublishContext = createContext<(chrome: PageChrome | null) => void>(
  () => undefined,
);

/* === Components === */

/**
 * Holds the page bar contents for the active route. The shell renders the
 * title and the action slot; pages publish them through `usePageChrome`.
 */
export function PageChromeProvider({ children }: { children: ReactNode }) {
  const [chrome, setChrome] = useState<PageChrome>(EMPTY_CHROME);
  const publish = useCallback((next: PageChrome | null) => {
    setChrome((current) => {
      const resolved = next ?? EMPTY_CHROME;
      return current.title === resolved.title && current.actions === resolved.actions
        ? current
        : resolved;
    });
  }, []);

  return (
    <PageChromePublishContext.Provider value={publish}>
      <PageChromeStateContext.Provider value={chrome}>{children}</PageChromeStateContext.Provider>
    </PageChromePublishContext.Provider>
  );
}

/* === Public Functions === */

/**
 * Publishes the page title and the one page-bar action for the mounted route.
 * `actions` is stored by identity, so callers must pass a `useMemo`-stable node
 * or the publish effect re-runs on every render.
 */
export function usePageChrome({ actions = null, title }: PageChromeInput): void {
  const publish = useContext(PageChromePublishContext);
  const chrome = useMemo<PageChrome>(() => ({ actions, title }), [actions, title]);

  useEffect(() => {
    publish(chrome);
    return () => publish(null);
  }, [chrome, publish]);
}

/** Reads the published chrome. Only the shell page bar needs this. */
export function usePageChromeState(): PageChrome {
  return useContext(PageChromeStateContext);
}
