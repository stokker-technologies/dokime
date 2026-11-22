// Dokime — SPDX-License-Identifier: Apache-2.0
import { useNativeSessionOptional } from "../app/native-session/context";
import type { ArtifactTreeProps } from "./ArtifactTreeView.svelte";
import NativeArtifactTree from "./NativeArtifactTree.svelte";
import { SvelteMount } from "./SvelteMount";

export function ArtifactTree(props: ArtifactTreeProps) {
  const session = useNativeSessionOptional();
  return <SvelteMount component={NativeArtifactTree} props={{ ...props, session }} />;
}
