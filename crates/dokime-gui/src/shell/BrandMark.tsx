// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ComponentProps } from "svelte";
import { SvelteMount } from "../components/SvelteMount";
import { useBrand } from "../lib/brand-context";
import BrandArtwork from "./BrandMark.svelte";

type Props = Omit<ComponentProps<typeof BrandArtwork>, "brand" | "class"> & { className?: string };

export function BrandMark({ className, ...props }: Props) {
  const brand = useBrand();
  return <SvelteMount component={BrandArtwork} props={{ ...props, brand, class: className }} />;
}
