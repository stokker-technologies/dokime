/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Remembered inputs belong to the operator, keyed by stable template and
   parameter IDs. Desktop persistence is native; browser previews remain local. */

/* === External Imports === */

import { z } from "zod";
import { create } from "zustand";
import { createJSONStorage, persist, type StateStorage } from "zustand/middleware";
import { toast } from "@/lib/notifications";

/* === Local Imports === */

import { commands } from "@/lib/desktop/commands";
import type { ParameterOverrides } from "@/lib/desktop/settings";
import { isTauriDesktop } from "@/lib/desktop-open";
import type { ParameterDefinition } from "@/lib/types";
import { isReportBuild } from "@/lib/ui-mode";

/* === Constants === */

export const PARAMETER_OVERRIDES_STORAGE_KEY = "dokime.parameter-overrides";

const parameterOverridesSchema = z.record(
  z.string(),
  z.record(z.string(), z.object({ raw: z.string(), valueType: z.string(), unit: z.string().nullable() })),
);

const persistedParameterOverridesSchema = z.object({
  overrides: parameterOverridesSchema,
});

const initialParameterOverridesState: ParameterOverridesStoreState = {
  overrides: {},
};

/* === Types === */

type ParameterOverridesStoreState = {
  overrides: ParameterOverrides;
};

type ParameterOverridesStore = ParameterOverridesStoreState & {
  clearTemplateOverrides: (templateId: string) => void;
  resetParameterOverride: (templateId: string, parameterId: string) => void;
  resetParameterOverridesStore: () => void;
  setParameterOverride: (templateId: string, definition: ParameterDefinition, raw: string) => void;
};

/* === Private State === */

let pending: ParameterOverrides | null = null;
let writing: Promise<void> | null = null;

export const useParameterOverridesStatus = create<{ ready: boolean; error: string | null }>(() => ({
  ready: false,
  error: null,
}));

/* === Public Functions === */

/** Refuse Unsupported Stored Values Without Replacing The Operator's Inputs. */
export function readPersistedParameterOverrides(value: unknown): ParameterOverrides {
  if (value === undefined) return {};
  return persistedParameterOverridesSchema.parse(value).overrides;
}

/** The override for one parameter, or `null` when the default still applies. */
export function parameterOverride(overrides: ParameterOverrides, templateId: string, parameterId: string): string | null {
  const raw = overrides[templateId]?.[parameterId];
  return raw?.raw ?? null;
}

export const useParameterOverridesStore = create<ParameterOverridesStore>()(
  persist(
    (set) => ({
      ...initialParameterOverridesState,

      clearTemplateOverrides: (templateId) =>
        set((state) => {
          if (!state.overrides[templateId]) {
            return state;
          }
          const next = { ...state.overrides };
          delete next[templateId];
          return { overrides: next };
        }),

      resetParameterOverride: (templateId, parameterId) =>
        set((state) => {
          const template = state.overrides[templateId];
          if (!template || !(parameterId in template)) {
            return state;
          }
          const nextTemplate = { ...template };
          delete nextTemplate[parameterId];
          const next = { ...state.overrides };
          if (Object.keys(nextTemplate).length) {
            next[templateId] = nextTemplate;
          } else {
            delete next[templateId];
          }
          return { overrides: next };
        }),

      resetParameterOverridesStore: () => set({ ...initialParameterOverridesState }),

      setParameterOverride: (templateId, definition, raw) =>
        set((state) => ({
          overrides: {
            ...state.overrides,
            [templateId]: {
              ...state.overrides[templateId],
              [definition.parameter_id]: {
                raw,
                valueType: (definition.value_type ?? "str").toLowerCase(),
                unit: definition.unit ?? null,
              },
            },
          },
        })),
    }),
    {
      merge: (persisted, current) => ({
        ...current,
        overrides: readPersistedParameterOverrides(persisted),
      }),
      name: PARAMETER_OVERRIDES_STORAGE_KEY,
      version: 1,
      migrate: () => {
        throw new Error("Unsupported remembered parameter version. Select fresh storage before editing.");
      },
      partialize: (state) => ({ overrides: state.overrides }),
      storage: createJSONStorage(parameterStorage),
      onRehydrateStorage: () => {
        useParameterOverridesStatus.setState({ ready: false, error: null });
        return (_state, error) => {
          useParameterOverridesStatus.setState({
            ready: !error,
            error: error ? `Could not load remembered parameters: ${String(error)}. Editing is disabled.` : null,
          });
        };
      },
    },
  ),
);

/* === Private Functions === */

function writeDesktop(values: ParameterOverrides): Promise<void> {
  pending = values;
  if (!writing) {
    writing = (async () => {
      while (pending) {
        const next = pending;
        pending = null;
        try {
          await commands.desktopParametersWrite(next);
        } catch (error) {
          toast.error("Could Not Save Remembered Parameters", { description: String(error) });
        }
      }
    })().finally(() => {
      writing = null;
      if (pending) return writeDesktop(pending);
    });
  }
  return writing;
}

function parameterStorage(): StateStorage {
  if (isReportBuild()) {
    return { getItem: () => null, setItem: () => undefined, removeItem: () => undefined };
  }
  if (!isTauriDesktop()) {
    return {
      getItem: (name) => window.localStorage.getItem(name),
      setItem: (name, value) => {
        if (useParameterOverridesStatus.getState().ready) window.localStorage.setItem(name, value);
      },
      removeItem: (name) => {
        if (useParameterOverridesStatus.getState().ready) window.localStorage.removeItem(name);
      },
    };
  }
  return {
    getItem: async () => {
      // Distinguish An Absent Native File From An Intentionally Empty Map.
      const values = await commands.desktopParametersRead();
      if (values === null) {
        if (window.localStorage.getItem(PARAMETER_OVERRIDES_STORAGE_KEY) !== null) {
          throw new Error("Unsupported browser-owned remembered parameters. Select fresh storage before editing.");
        }
        return null;
      }
      return JSON.stringify({ state: { overrides: values }, version: 1 });
    },
    setItem: (_name, payload) => {
      if (!useParameterOverridesStatus.getState().ready) return;
      return writeDesktop(readPersistedParameterOverrides(JSON.parse(payload).state));
    },
    removeItem: () => {
      if (!useParameterOverridesStatus.getState().ready) return;
      return writeDesktop({});
    },
  };
}
