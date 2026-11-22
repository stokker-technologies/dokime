// Dokime — SPDX-License-Identifier: Apache-2.0
import { useNativeSessionOptional } from "../app/native-session/context";
import MediaGalleryView from "./MediaGalleryView.svelte";
import type { MediaGalleryProps } from "./media-gallery-model";
import { nativeMediaTransfers } from "./native-media-transfers";
import { SvelteMount } from "./SvelteMount";

export function MediaGallery(props: MediaGalleryProps) {
  const session = useNativeSessionOptional();
  const transfers = session ? nativeMediaTransfers(session) : undefined;
  return <SvelteMount component={MediaGalleryView} props={{ ...props, transfers }} />;
}
