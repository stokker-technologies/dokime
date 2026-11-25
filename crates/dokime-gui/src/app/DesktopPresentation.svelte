<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { Brand } from "../lib/brand";
  import type { DensityPreference, ThemeModePreference } from "../lib/theme";
  import type { ReportDefaults } from "../plots/plot-settings";

  export type Presentation = {
    brand: Brand;
    desktop: boolean;
    theme: ThemeModePreference;
    densityDisabled: boolean;
    themeDisabled: boolean;
    chooseDensity: (density: DensityPreference) => void;
    chooseTheme: (mode: ThemeModePreference) => void;
    reportDefaults: ReportDefaults;
    plotPalette: string[];
  };

  const THEME_STORAGE_KEY = "dokime-test-hub.theme";
  let reportedOrganizationError = false;
</script>

<script lang="ts">
  import type { ApplicationSettings, PreferencesSnapshot } from "@generated/typescript/application-settings";
  import { createMutation, createQuery, type QueryClient, QueryClientProvider } from "@tanstack/svelte-query";
  import { onMount, type Snippet } from "svelte";
  import DokimeToaster from "../components/DokimeToaster.svelte";
  import { provideErrorRecovery } from "../components/error-recovery";
  import { preferencesMutation, preferencesQuery } from "../features/settings/application-preferences";
  import { type AppearanceSnapshot, bootOrganization } from "../lib/brand";
  import { setDesktopWindowTitle } from "../lib/desktop-title";
  import { failureRecovery } from "../lib/failure-recovery";
  import { applyBrandFavicon } from "../lib/favicon";
  import { failureOf } from "../lib/notification-details";
  import { toast } from "../lib/notifications";
  import { isTauriDesktop } from "../lib/surface-environment";
  import { applyDensity, applyThemeStylesheet, type ResolvedTheme } from "../lib/theme";
  import { resolvePlotThemeColors } from "../plots/colors";
  import { BUILT_IN_REPORT_DEFAULTS } from "../plots/plot-settings";
  import { storeState } from "../stores/store-state";
  import { uiStore } from "../stores/ui-store";
  import { desktopAppearance } from "./desktop-appearance";
  import type { ConnectedSession } from "./native-session/session";

  let {
    session,
    client,
    resource,
    children,
  }: {
    session: ConnectedSession | null;
    client: QueryClient;
    resource: AppearanceSnapshot | null;
    children: Snippet<[Presentation]>;
  } = $props();

  provideErrorRecovery((failure) => failureRecovery(session, failure));
  const desktop = isTauriDesktop();
  const local = createQuery(
    () => ({ ...preferencesQuery("local", session), enabled: desktop }),
    () => client,
  );
  const backend = createQuery(
    () => ({ ...preferencesQuery("backend", session), enabled: desktop && !!session }),
    () => client,
  );
  const save = createMutation(
    () => preferencesMutation("local", session, client),
    () => client,
  );
  const identity = $derived(desktopAppearance(resource));
  const density = storeState(uiStore, (state) => state.density);
  let browserTheme = $state<ThemeModePreference>("system");
  let system = $state<ResolvedTheme>("light");
  const current = $derived(local.data?.settings.appearance);
  const theme = $derived(desktop ? (current?.theme_mode ?? "system") : browserTheme);
  const resolved = $derived(theme === "system" ? system : theme);
  let plotPalette = $state.raw(resolvePlotThemeColors().palette);
  const report = $derived(backend.data?.settings.report);
  const reportDefaults = $derived(
    report
      ? {
          plotPoints: report.plot_points,
          plotDownsample: report.plot_downsample === "unspecified" ? BUILT_IN_REPORT_DEFAULTS.plotDownsample : report.plot_downsample,
        }
      : BUILT_IN_REPORT_DEFAULTS,
  );

  onMount(() => {
    if (!desktop) {
      const saved = window.localStorage.getItem(THEME_STORAGE_KEY);
      if (saved === "light" || saved === "dark") {
        browserTheme = saved;
      }
    }
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const change = () => (system = media.matches ? "dark" : "light");
    change();
    media.addEventListener("change", change);
    const error = bootOrganization.error;
    if (error && !reportedOrganizationError) {
      reportedOrganizationError = true;
      toast.error(failureOf(error, "Organization Pack Not Applied"), { context: "background" });
    }
    return () => media.removeEventListener("change", change);
  });

  $effect(() => {
    applyBrandFavicon(identity.brand);
    document.title = identity.title;
    void setDesktopWindowTitle(identity.title);
  });
  $effect(() => {
    applyThemeStylesheet(identity.stylesheet);
    return () => applyThemeStylesheet(null);
  });
  // Plots Read the Palette the Applied Stylesheet and Mode Resolve To.
  $effect(() => {
    void identity.stylesheet;
    const root = document.documentElement;
    root.classList.toggle("dark", resolved === "dark");
    root.classList.toggle("light", resolved === "light");
    root.style.colorScheme = resolved;
    plotPalette = resolvePlotThemeColors().palette;
  });
  $effect(() => {
    if (current) {
      uiStore.getState().setUiState({ density: current.density, themeFamily: current.theme_family });
    }
  });
  $effect(() => {
    if (session) {
      applyDensity(density());
    }
  });

  function putAppearance(patch: Partial<ApplicationSettings["appearance"]>, previous?: PreferencesSnapshot) {
    if (!previous) {
      toast.error(failureOf("Reload local preferences before saving appearance.", "Appearance Could Not Be Saved"));
      return;
    }
    save.mutate(
      { previous, settings: { ...previous.settings, appearance: { ...previous.settings.appearance, ...patch } } },
      {
        onError: (error) => toast.error(failureOf(error, "Appearance Could Not Be Saved")),
      },
    );
  }

  function chooseDensity(density: DensityPreference) {
    if (desktop) {
      putAppearance({ density }, local.data);
    } else {
      uiStore.getState().setUiState({ density });
    }
  }

  function chooseTheme(theme_mode: ThemeModePreference) {
    if (desktop) {
      putAppearance({ theme_mode }, local.data);
    } else {
      browserTheme = theme_mode;
      window.localStorage.setItem(THEME_STORAGE_KEY, theme_mode);
    }
  }
</script>

<QueryClientProvider {client}>
  {@render children({
    brand: identity.brand,
    desktop,
    theme,
    plotPalette,
    reportDefaults,
    chooseDensity,
    chooseTheme,
    densityDisabled: desktop && (!local.data || save.isPending),
    themeDisabled: save.isPending,
  })}
  <DokimeToaster theme={resolved} />
</QueryClientProvider>
