// Dokime — SPDX-License-Identifier: Apache-2.0
import { useApplicationTheme } from "../lib/application-theme";
import Toaster from "./DokimeToaster.svelte";
import { SvelteMount } from "./SvelteMount";

export function DokimeToaster() {
  const { resolvedTheme } = useApplicationTheme();
  return <SvelteMount component={Toaster} props={{ theme: resolvedTheme === "dark" ? "dark" : "light" }} />;
}
