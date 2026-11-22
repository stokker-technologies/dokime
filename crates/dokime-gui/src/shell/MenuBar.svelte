<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Menubar } from "bits-ui";
  import { Check } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import type { DensityPreference, ThemeModePreference } from "../lib/theme";

  type Props = {
    connected: boolean;
    local: boolean;
    desktop: boolean;
    focused: boolean;
    focusDisabled: boolean;
    consoleOpen: boolean;
    density: DensityPreference;
    densityDisabled: boolean;
    theme: ThemeModePreference;
    themeDisabled: boolean;
    newCycle: () => void;
    settings: () => void;
    disconnect: () => void;
    quit: () => void;
    toggleSidebar: () => void;
    setFocused: (next: boolean) => void;
    setConsoleOpen: (next: boolean) => void;
    setDensity: (next: DensityPreference) => void;
    setTheme: (next: ThemeModePreference) => void;
    diagnostics: () => void;
    about: () => void;
  };

  let props: Props = $props();
  const densities = [
    { value: "comfortable", label: "Comfortable" },
    { value: "compact", label: "Compact" },
    { value: "touch", label: "Touch" },
  ] as const;
  const themes = [
    { value: "system", label: "System" },
    { value: "light", label: "Light" },
    { value: "dark", label: "Dark" },
  ] as const;

  const triggerClass =
    "flex h-full items-center rounded-none px-3 py-1 font-normal [font-size:inherit] outline-hidden select-none hover:bg-muted aria-expanded:bg-muted";
  const itemClass =
    "relative flex cursor-default items-center gap-2 rounded-lg px-2 py-1 text-sm outline-hidden select-none data-highlighted:bg-accent data-highlighted:text-accent-foreground data-disabled:pointer-events-none data-disabled:opacity-50";
  const choiceClass = `${itemClass} pl-6 max-lg:min-h-(--control-height) [[data-density=touch]_&]:min-h-(--control-height)`;
  const labelClass = "px-2 py-1 text-xs font-medium text-muted-foreground";

  function chooseDensity(next: string) {
    const option = densities.find((entry) => entry.value === next);
    if (option) {
      props.setDensity(option.value);
    }
  }

  function chooseTheme(next: string) {
    const option = themes.find((entry) => entry.value === next);
    if (option) {
      props.setTheme(option.value);
    }
  }
</script>

{#snippet mark()}
  <Check aria-hidden="true" class="pointer-events-none absolute left-1.5 size-4" />
{/snippet}

{#snippet separator()}
  <Menubar.Separator data-slot="menubar-separator" class="-mx-1 my-1 h-px bg-border" />
{/snippet}

{#snippet menu(
  value: string,
  label: string,
  items: Snippet,
)}
  <Menubar.Menu {value}>
    <Menubar.Trigger data-slot="menubar-trigger" class={triggerClass}>{label}</Menubar.Trigger>
    <Menubar.Portal>
      <Menubar.Content
        data-slot="menubar-content"
        align="start"
        alignOffset={-4}
        sideOffset={8}
        class="z-50 min-w-36 overflow-hidden rounded-lg bg-popover p-1 text-popover-foreground ring-1 ring-border duration-100 data-open:animate-in data-open:fade-in-0"
      >
        {@render items()}
      </Menubar.Content>
    </Menubar.Portal>
  </Menubar.Menu>
{/snippet}

{#snippet item(
  label: string,
  select: () => void,
  disabled = false,
)}
  <Menubar.Item data-slot="menubar-item" class={itemClass} onSelect={select} {disabled}>{label}</Menubar.Item>
{/snippet}

{#snippet checkItem(
  label: string,
  checked: boolean,
  change: (next: boolean) => void,
  disabled = false,
)}
  <Menubar.CheckboxItem data-slot="menubar-checkbox-item" class={choiceClass} {checked} {disabled} onCheckedChange={change}>
    {#if checked}
      {@render mark()}
    {/if}
    {label}
  </Menubar.CheckboxItem>
{/snippet}

{#snippet radioItems(
  label: string,
  options: readonly { value: string; label: string }[],
  value: string,
  change: (next: string) => void,
  disabled: boolean,
)}
  <Menubar.Group>
    <Menubar.GroupHeading data-slot="menubar-label" class={labelClass}>{label}</Menubar.GroupHeading>
    <Menubar.RadioGroup data-slot="menubar-radio-group" {value} onValueChange={change}>
      {#each options as option (option.value)}
        <Menubar.RadioItem data-slot="menubar-radio-item" class={choiceClass} {disabled} value={option.value}>
          {#if value === option.value}
            {@render mark()}
          {/if}
          {option.label}
        </Menubar.RadioItem>
      {/each}
    </Menubar.RadioGroup>
  </Menubar.Group>
{/snippet}

{#snippet file()}
  {@render item("New Cycle", props.newCycle, !props.connected)}
  {@render separator()}
  {@render item("Settings…", props.settings)}
  {#if props.connected || props.desktop}
    {@render separator()}
  {/if}
  {#if props.connected}
    {@render item(props.local ? "Stop Local Backend" : "Disconnect", props.disconnect)}
  {/if}
  {#if props.desktop}
    {@render item("Quit", props.quit)}
  {/if}
{/snippet}

{#snippet view()}
  {@render item("Toggle Sidebar", props.toggleSidebar)}
  {@render checkItem("Focus Mode", props.focused, props.setFocused, props.focusDisabled)}
  {@render checkItem("Console", props.consoleOpen, props.setConsoleOpen)}
  {@render separator()}
  {@render radioItems("Density", densities, props.density, chooseDensity, props.densityDisabled)}
  {@render separator()}
  {@render radioItems("Theme", themes, props.theme, chooseTheme, props.themeDisabled)}
{/snippet}

{#snippet help()}
  {@render item("Diagnostics", props.diagnostics)}
  {@render item("About", props.about)}
{/snippet}

<Menubar.Root
  aria-label="Application"
  data-slot="menubar"
  class="flex h-[var(--menu-bar-height)] items-center gap-0 rounded-none border-0 border-b bg-background p-0 px-[calc(var(--shell-inset)/2)]"
>
  {@render menu("file", "File", file)}
  {@render menu("view", "View", view)}
  {@render menu("help", "Help", help)}
</Menubar.Root>
