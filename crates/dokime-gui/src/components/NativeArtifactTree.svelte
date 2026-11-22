<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { ConnectedSession } from "../app/native-session/session";
  import { downloadNativeArtifact, nativeContentSession, openNativeArtifact } from "../lib/api/native-content";
  import type { ArtifactTreeFile } from "../lib/artifact-tree";
  import ArtifactTreeView, { type ArtifactTreeProps } from "./ArtifactTreeView.svelte";
  import TextArtifactViewer from "./TextArtifactViewer.svelte";

  type Props = ArtifactTreeProps & { session: ConnectedSession | null };
  let { session, ...tree }: Props = $props();
  let preview = $state<ArtifactTreeFile | null>(null);
  const content = $derived(session ? nativeContentSession(session.identity, session.own) : null);
  const actions = $derived.by(() => {
    const current = content;
    return current
      ? {
          open: (artifact: ArtifactTreeFile) => openNativeArtifact(current, artifact),
          download: (artifact: ArtifactTreeFile) => downloadNativeArtifact(current, artifact),
        }
      : undefined;
  });
</script>

<ArtifactTreeView
  {...tree}
  {actions}
  onPreview={(artifact) => {
    preview = artifact;
  }}
/>
<TextArtifactViewer
  artifact={preview}
  {session}
  open={preview !== null}
  onOpenChange={(open) => {
    if (!open) {
      preview = null;
    }
  }}
/>
