/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import type { ReactNode } from "react";

/* === Local Imports === */

import { Label } from "@/components/ui/label";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsFieldStatus } from "@/features/settings/SettingsFieldStatus";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { BrandingPackRow } from "@/features/settings/sections/BrandingPackRow";
import { useAutosave } from "@/features/settings/settings-autosave";
import { useAppearance } from "@/lib/appearance";
import { type Appearance, applicationSettingsError, useApplicationSettings, useSaveAppearance } from "@/lib/application-settings";
import { useApplicationTheme } from "@/lib/application-theme";
import { useBrand } from "@/lib/brand";
import { isTauriDesktop } from "@/lib/desktop-open";
import type { DensityPreference, ThemeModePreference } from "@/lib/theme";
import { resolveThemeFamily } from "@/lib/theme-family";
import { BrandMark } from "@/shell/BrandMark";
import { useUiStore } from "@/stores/ui-store";

/* === Constants === */

const MODE_OPTIONS: { value: ThemeModePreference; label: string }[] = [
  { value: "system", label: "System" },
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
];

/* === Components === */

export function AppearanceSection() {
  return isTauriDesktop() ? <DesktopAppearanceSection /> : <AppearanceControls />;
}

function DesktopAppearanceSection() {
  const query = useApplicationSettings();
  const save = useSaveAppearance();
  const autosave = useAutosave<Partial<Appearance>>((patch) => save.mutateAsync(patch), applicationSettingsError);

  return (
    <AppearanceControls
      disabled={!query.data}
      loadError={query.error ? applicationSettingsError(query.error) : null}
      save={autosave.commit}
      status={<SettingsFieldStatus onRetry={autosave.retry} state={autosave.state} />}
    />
  );
}

function AppearanceControls({
  disabled = false,
  loadError = null,
  save,
  status = null,
}: {
  disabled?: boolean;
  loadError?: string | null;
  save?: (patch: Partial<Appearance>) => void;
  status?: ReactNode;
}) {
  const appearance = useAppearance();
  const { forcedTheme, setTheme, theme, saving } = useApplicationTheme();
  const density = useUiStore((state) => state.density);
  const themeFamily = useUiStore((state) => state.themeFamily);
  const setUiState = useUiStore((state) => state.setUiState);
  const brand = useBrand();
  const selectedThemeFamily = resolveThemeFamily(appearance.themes, {
    activeThemeId: appearance.active_theme,
    storedThemeId: themeFamily,
  })?.theme_id;
  const showThemeFamilySelector = !appearance.theme_family_locked && appearance.themes.length > 1;
  const modeControlDisabled = disabled || saving || appearance.mode_locked || Boolean(forcedTheme);

  return (
    <>
      <SettingsCard title="Appearance">
        {loadError ? (
          <SettingsRow label="Status">
            <span className="text-sm text-destructive" role="alert">
              {loadError}
            </span>
          </SettingsRow>
        ) : null}
        <SettingsRow
          description="Follow the operating system, or force light or dark."
          label="Mode"
          status={
            saving ? (
              <span className="text-xs text-muted-foreground" role="status">
                Saving
              </span>
            ) : (
              status
            )
          }
        >
          <RadioGroup
            className="flex flex-col items-start gap-2 sm:flex-row sm:items-center sm:gap-4"
            onValueChange={(value) => setTheme(value as ThemeModePreference)}
            value={theme ?? "system"}
          >
            {MODE_OPTIONS.map((option) => (
              <div className="flex items-center gap-2" key={option.value}>
                <RadioGroupItem disabled={modeControlDisabled} id={`appearance-mode-${option.value}`} value={option.value} />
                <Label htmlFor={`appearance-mode-${option.value}`}>{option.label}</Label>
              </div>
            ))}
          </RadioGroup>
        </SettingsRow>
        {showThemeFamilySelector && (
          <SettingsRow description="Choose a registered theme family." label="Theme Family" status={status}>
            <Select
              disabled={disabled}
              onValueChange={(value) => (save ? save({ themeFamily: value }) : setUiState({ themeFamily: value }))}
              value={selectedThemeFamily}
            >
              <SelectTrigger className="w-32 sm:w-56">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {appearance.themes.map((theme) => (
                  <SelectItem key={theme.theme_id} value={theme.theme_id}>
                    {theme.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </SettingsRow>
        )}
        <SettingsRow
          description="Compact fits more on screen; touch enlarges every control for gloves and tablets."
          label="Density"
          status={status}
        >
          <ToggleGroup
            className="flex-wrap justify-end"
            disabled={disabled}
            onValueChange={(value) => {
              if (value) {
                if (save) save({ density: value as DensityPreference });
                else setUiState({ density: value as DensityPreference });
              }
            }}
            type="single"
            value={density}
          >
            <ToggleGroupItem value="comfortable">Comfortable</ToggleGroupItem>
            <ToggleGroupItem value="compact">Compact</ToggleGroupItem>
            <ToggleGroupItem value="touch">Touch</ToggleGroupItem>
          </ToggleGroup>
        </SettingsRow>
      </SettingsCard>

      <SettingsCard title="Brand">
        <SettingsRow description={appearance.organization?.name ?? `The identity shown in the shell.`} label="Logo And Name">
          <div className="flex items-center gap-2">
            <BrandMark />
            <span className="font-heading text-sm font-medium">{brand.label}</span>
          </div>
        </SettingsRow>
        {isTauriDesktop() ? <BrandingPackRow /> : null}
      </SettingsCard>
    </>
  );
}
