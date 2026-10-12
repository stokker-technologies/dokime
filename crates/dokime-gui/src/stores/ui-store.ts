/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { z } from "zod";
import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

/* === Local Imports === */

import { isTauriDesktop } from "@/lib/desktop-open";
import type { DensityPreference } from "@/lib/theme";

/* === Types === */

/**
 * The console drawer's own state.
 *
 * `Trace` is a threshold, not a severity the backend emits: the projection
 * carries four levels, and asking for Trace means "hold nothing back".
 */
export type ConsoleLevel = "trace" | "debug" | "info" | "warning" | "error";

export type ConsoleScope = "instance" | "cycle" | "execution";

export type ConsoleState = {
  /** Auto-scroll to the newest row. */
  follow: boolean;
  /** Drawer height in pixels, dragged by the operator. */
  height: number;
  level: ConsoleLevel;
  /** Wall-clock ms the drawer was last opened; the badge counts from here. */
  lastOpenedAt: number;
  open: boolean;
  scope: ConsoleScope;
};

export type AppNotificationTone = "error" | "info" | "loading" | "success" | "warning";

export type AppNotificationRecord = {
  /** Durable action target (e.g. a download href) rendered in the notification center. */
  actionHref?: string;
  actionLabel?: string;
  /** Wall-clock ms when the notification was raised. */
  at?: number;
  description?: string;
  id: string;
  title: string;
  tone: AppNotificationTone;
};

type UiStoreState = {
  console: ConsoleState;
  cycleSummaryCardBusy: boolean;
  cycleSummaryCardDescription: string;
  cycleSummaryCardDialogOpen: boolean;
  cycleSummaryCardMarkdown: string;
  cycleSummaryCardTitle: string;
  clearDialogOpen: boolean;
  clearCycleError: string | null;
  clearingCycle: boolean;
  density: DensityPreference;
  desktopUpdateDialogOpen: boolean;
  leftRailCollapsed: boolean;
  noteText: string;
  notificationLog: AppNotificationRecord[];
  rightRailCollapsed: boolean;
  railCollapseMode: "auto" | "manual";
  /**
   * The contextual sidebar's collapsed state, one answer per density.
   *
   * A tablet operator and a desk operator are the same person with different
   * room: the choice made at one density must not follow them to the other.
   */
  sidebarCollapsed: Partial<Record<DensityPreference, boolean>>;
  themeFamily: string | null;
  traceabilityLocation: string;
  traceabilityLocationDraft: string;
};

type UiStore = UiStoreState & {
  clearNotifications: () => void;
  setConsoleState: (patch: Partial<ConsoleState>) => void;
  pushNotification: (notification: AppNotificationRecord) => void;
  removeNotification: (id: string) => void;
  resetUiStore: () => void;
  setUiState: (patch: Partial<UiStoreState>) => void;
};

/* === Constants === */

export const INITIAL_CONSOLE_STATE: ConsoleState = {
  follow: true,
  height: 240,
  lastOpenedAt: 0,
  level: "info",
  open: false,
  scope: "cycle",
};

const initialUiStoreState: UiStoreState = {
  console: INITIAL_CONSOLE_STATE,
  cycleSummaryCardBusy: false,
  cycleSummaryCardDescription: "",
  cycleSummaryCardDialogOpen: false,
  cycleSummaryCardMarkdown: "",
  cycleSummaryCardTitle: "Cycle Summary Note",
  clearDialogOpen: false,
  clearCycleError: null,
  clearingCycle: false,
  density: "comfortable",
  desktopUpdateDialogOpen: false,
  leftRailCollapsed: false,
  noteText: "",
  notificationLog: [],
  rightRailCollapsed: false,
  railCollapseMode: "auto",
  sidebarCollapsed: {},
  themeFamily: null,
  traceabilityLocation: "",
  traceabilityLocationDraft: "",
};

const persistedConsoleSchema = z
  .object({
    follow: z.boolean().catch(INITIAL_CONSOLE_STATE.follow),
    height: z.number().min(120).max(900).catch(INITIAL_CONSOLE_STATE.height),
    level: z
      .enum(["trace", "debug", "info", "warning", "error"])
      .catch(INITIAL_CONSOLE_STATE.level),
    scope: z.enum(["instance", "cycle", "execution"]).catch(INITIAL_CONSOLE_STATE.scope),
  })
  .catch({
    follow: INITIAL_CONSOLE_STATE.follow,
    height: INITIAL_CONSOLE_STATE.height,
    level: INITIAL_CONSOLE_STATE.level,
    scope: INITIAL_CONSOLE_STATE.scope,
  });

const persistedUiStateSchema = z.object({
  console: persistedConsoleSchema.default({
    follow: INITIAL_CONSOLE_STATE.follow,
    height: INITIAL_CONSOLE_STATE.height,
    level: INITIAL_CONSOLE_STATE.level,
    scope: INITIAL_CONSOLE_STATE.scope,
  }),
  density: z.enum(["comfortable", "compact", "touch"]).catch("comfortable"),
  sidebarCollapsed: z
    .object({
      comfortable: z.boolean().optional(),
      compact: z.boolean().optional(),
      touch: z.boolean().optional(),
    })
    .catch({})
    .default({}),
  themeFamily: z.string().nullable().optional(),
});

/* === Private Functions === */

/** Under Tauri the WebView copy is a sidebar-only sidecar of the store's key. */
function storageKey(name: string): string {
  return isTauriDesktop() ? `${name}.sidebar` : name;
}

/* === Public Functions === */

/** Collapsed unless the operator said otherwise; a touch surface starts narrow. */
export function sidebarStartsCollapsed(
  collapsed: Partial<Record<DensityPreference, boolean>>,
  density: DensityPreference,
): boolean {
  return collapsed[density] ?? density === "touch";
}

export const useUiStore = create<UiStore>()(
  persist(
    (set) => ({
      ...initialUiStoreState,
      clearNotifications: () => set({ notificationLog: [] }),
      setConsoleState: (patch) => set((state) => ({ console: { ...state.console, ...patch } })),
      pushNotification: (notification) =>
        set((state) => ({
          notificationLog: [
            ...state.notificationLog.filter((item) => item.id !== notification.id),
            { at: Date.now(), ...notification },
          ].slice(-50),
        })),
      removeNotification: (id) =>
        set((state) => ({
          notificationLog: state.notificationLog.filter((notification) => notification.id !== id),
        })),
      resetUiStore: () =>
        set((state) => ({
          ...initialUiStoreState,
          density: state.density,
          themeFamily: state.themeFamily,
        })),
      setUiState: (patch) => set((state) => ({ ...state, ...patch })),
    }),
    {
      merge: (persisted, current) => {
        const parsed = persistedUiStateSchema.safeParse(
          (persisted as { density?: unknown; theme?: unknown }) ?? {},
        );
        if (!parsed.success) {
          return current;
        }
        // Desktop settings own density and the theme family under Tauri, so a
        // stale local copy must not win the first frame. The sidebar has no
        // backend home, so its collapsed state is restored on both surfaces.
        const desktop = isTauriDesktop();
        return {
          ...current,
          console: { ...INITIAL_CONSOLE_STATE, ...parsed.data.console },
          density: desktop ? current.density : parsed.data.density,
          sidebarCollapsed: parsed.data.sidebarCollapsed,
          themeFamily: desktop ? current.themeFamily : (parsed.data.themeFamily ?? null),
        };
      },
      name: "dokime-test-hub.ui",
      partialize: (state) => ({
        console: {
          follow: state.console.follow,
          height: state.console.height,
          level: state.console.level,
          scope: state.console.scope,
        },
        density: state.density,
        sidebarCollapsed: state.sidebarCollapsed,
        themeFamily: state.themeFamily,
      }),
      /* Desktop settings own density and the theme family, so under Tauri the
         WebView keeps only what has no backend home: the sidebar and the
         console drawer's geometry. In a browser
         there is no other home for any of it, so the whole slice is kept. */
      storage: createJSONStorage(() => ({
        getItem: (name) => window.localStorage.getItem(storageKey(name)),
        removeItem: (name) => window.localStorage.removeItem(storageKey(name)),
        setItem: (name, value) => {
          if (!isTauriDesktop()) {
            window.localStorage.setItem(name, value);
            return;
          }
          const state = (JSON.parse(value) as { state?: UiStoreState }).state;
          window.localStorage.setItem(
            storageKey(name),
            JSON.stringify({
              state: {
                console: state?.console ?? INITIAL_CONSOLE_STATE,
                sidebarCollapsed: state?.sidebarCollapsed ?? {},
              },
            }),
          );
        },
      })),
    },
  ),
);
