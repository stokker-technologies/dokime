<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import DocumentWorkspacePresentation from "../../components/document/DocumentWorkspacePresentation.svelte";
  import { resolvePlotThemeColors } from "../../plots/colors";
  import { documentWorkspace, type DocumentViewProps } from "./document-workspace";

  let { onRetry, ...props }: DocumentViewProps & { onRetry?: (cardId?: string) => void } = $props();
  const workspace = $derived(documentWorkspace(props));
  const plotPalette = resolvePlotThemeColors().palette;
</script>

<DocumentWorkspacePresentation {...workspace} {onRetry} {plotPalette} />
