// Dokime — SPDX-License-Identifier: Apache-2.0
import { createSubscriber } from "svelte/reactivity";
import type { CycleSelection } from "../features/cycle/cycle-workspace-model";
import type { ExploreTarget } from "../features/explore/explore-contracts";
import { type FocusMode, focusSearchSchema } from "../shell/focus-mode";

const locationChanged = createSubscriber((update) => {
  window.addEventListener("popstate", update);
  window.addEventListener("hashchange", update);
  return () => {
    window.removeEventListener("popstate", update);
    window.removeEventListener("hashchange", update);
  };
});

export function readLocation(): URL {
  locationChanged();
  return new URL(window.location.href);
}

type Search = Readonly<Record<string, string | undefined>>;

export function navigate(path: string, search?: Search, replace = false): void {
  const next = new URL(path, window.location.href);
  for (const [key, value] of Object.entries(search ?? {})) {
    if (value === undefined) {
      next.searchParams.delete(key);
    } else {
      next.searchParams.set(key, value);
    }
  }
  if (next.href === window.location.href) {
    return;
  }
  window.history[replace ? "replaceState" : "pushState"](null, "", next);
  window.dispatchEvent(new PopStateEvent("popstate"));
}

export function optionalSearch(url: URL, key: string): string | undefined {
  return url.searchParams.getAll(key).at(-1)?.trim() || undefined;
}

export function pathParts(url: URL): string[] {
  try {
    return url.pathname.split("/").filter(Boolean).map(decodeURIComponent);
  } catch {
    return [];
  }
}

export function cyclePath(cycleId: string): string {
  return `/cycles/${encodeURIComponent(cycleId)}`;
}

export function libraryPath(cycleId: string): string {
  return `/library?${new URLSearchParams({ appendTo: cycleId })}`;
}

export function pluginPath(id: string): string {
  return `/settings/plugins/${encodeURIComponent(id)}`;
}

export function historyPath({ id, kind }: ExploreTarget): string {
  return `/history/${kind === "execution" ? "executions/" : ""}${encodeURIComponent(id)}`;
}

export function cycleSelection(url: URL): CycleSelection {
  return {
    caseId: optionalSearch(url, "caseId"),
    executionId: optionalSearch(url, "executionId"),
    view: optionalSearch(url, "view") === "cycle" ? "cycle" : undefined,
  };
}

export function focusMode(url: URL, cycleId: string | undefined): FocusMode {
  const value = focusSearchSchema.parse(url.searchParams.getAll("focus"));
  return {
    focused: Boolean(cycleId && value),
    setFocused(next) {
      if (cycleId) {
        navigate(url.pathname + url.search, { focus: next ? "1" : undefined }, true);
      }
    },
  };
}
