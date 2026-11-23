<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { defaultBrand, selectLogo, type Brand } from "../lib/brand";
  import { cn } from "../lib/utils";

  type Props = {
    brand?: Brand;
    class?: string;
    heading?: boolean;
    logoOnly?: boolean;
    treatment?: "transparent" | "contained";
  };

  let { brand = defaultBrand, class: style, heading = false, logoOnly = false, treatment = "transparent" }: Props = $props();
  const surfaces = ["light", "dark"] as const;
  const artwork = $derived(selectLogo(brand, treatment, "light") ?? selectLogo(brand, treatment, "dark"));
</script>

<div class={cn("brand brand-row brand-center", style)}>
  {#each surfaces as surface (surface)}
    {@const logo = selectLogo(brand, treatment, surface)}
    {@const visibility = surface === "light" ? "dark:hidden" : "hidden dark:block"}
    {#if logo}
      <img alt={logo.alt ?? brand.label} class={cn("logo", visibility)} src={logo.src}>
    {:else if artwork && logoOnly}
      <span class={cn("logo", visibility)}>{brand.label}</span>
    {/if}
  {/each}
  {#if !logoOnly || !artwork}
    <svelte:element this={heading ? "h1" : "span"} class={cn("wordmark", !artwork && "logo")}>{brand.label}</svelte:element>
  {/if}
</div>
