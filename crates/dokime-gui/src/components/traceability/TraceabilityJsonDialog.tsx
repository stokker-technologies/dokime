// Dokime — SPDX-License-Identifier: Apache-2.0
import { createContext, useContext } from "react";
import type { ComponentProps } from "svelte";
import { SvelteMount } from "../SvelteMount";
import TraceabilityJsonDialogView from "./TraceabilityJsonDialog.svelte";

export const TraceabilitySaveContext = createContext<((json: string, filename: string) => Promise<void>) | null>(null);

export function TraceabilityJsonDialog(props: Omit<ComponentProps<typeof TraceabilityJsonDialogView>, "save">) {
  const save = useContext(TraceabilitySaveContext);
  return <SvelteMount component={TraceabilityJsonDialogView} props={{ ...props, save }} />;
}
