<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { FileUp, Loader2 } from "lucide-svelte";
  import { onMount } from "svelte";
  import ErrorCard from "../../../components/ErrorCard.svelte";
  import Button from "../../../components/ui/Button.svelte";
  import { formatInstallPluginBytes, type InstallPluginProgress } from "../../../lib/api/native-plugins";
  import { failureOf } from "../../../lib/notification-details";
  import { toast } from "../../../lib/notifications";
  import { isTauriDesktop } from "../../../lib/surface-environment";

  let {
    label,
    disabled,
    failure,
    filename,
    onDrop,
    onPick,
    progress,
  }: {
    label: string;
    disabled: boolean;
    failure?: unknown;
    filename: string | null;
    onDrop: (filename: string) => void;
    onPick: () => void;
    progress: InstallPluginProgress | null;
  } = $props();
  let zone: HTMLFieldSetElement;
  let dragging = $state(false);

  // Native File Drops Stay In The Shell, Which Supplies The File To Install.
  onMount(() => {
    if (!isTauriDesktop()) {
      return;
    }
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void getCurrentWebview()
      .onDragDropEvent(({ payload }) => {
        if (payload.type === "drop" && payload.paths.some((path) => path.endsWith(".pair"))) {
          return;
        }
        if (payload.type === "leave") {
          dragging = false;
          return;
        }
        const bounds = zone.getBoundingClientRect();
        const x = payload.position.x / window.devicePixelRatio;
        const y = payload.position.y / window.devicePixelRatio;
        const hovering = !disabled && x >= bounds.left && x <= bounds.right && y >= bounds.top && y <= bounds.bottom;
        dragging = payload.type !== "drop" && hovering;
        if (payload.type === "drop" && hovering) {
          onDrop(payload.paths[0]?.split(/[\\/]/).at(-1) ?? "plugin");
        }
      })
      .then((stop) => {
        if (disposed) {
          stop();
        } else {
          unlisten = stop;
        }
      })
      .catch((error: unknown) => {
        if (!disposed) {
          toast.error(failureOf(error, "Plugin File Drop Unavailable"), { context: "background" });
        }
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  });

  function installationLabel(): string {
    switch (progress?.phase) {
      case undefined:
        return `Installing ${filename ?? "plugin"}…`;
      case "reading":
        return "Reading File…";
      case "uploading":
        return `Uploading ${formatInstallPluginBytes(progress.sent)} of ${formatInstallPluginBytes(progress.total)}`;
      case "installing":
        return "Installing on the Host…";
    }
  }
</script>

<div class="grid gap-2">
  <fieldset
    aria-busy={disabled}
    aria-label="Plugin File Drop Zone"
    {disabled}
    bind:this={zone}
    class={[
      "grid min-h-40 place-items-center rounded-lg border border-dashed bg-muted/20 p-6 text-center transition-colors",
      dragging && "border-primary bg-primary/5",
      disabled && "opacity-70",
    ]}
  >
    <div class="grid max-w-md justify-items-center gap-3">
      {#if disabled}
        <Loader2 class="size-8 animate-spin text-muted-foreground" />
      {:else}
        <FileUp class="size-8 text-muted-foreground" />
      {/if}
      <div class="grid gap-1">
        <p class="font-medium">{disabled ? installationLabel() : "Drop a Plugin Bundle Here"}</p>
        <p class="text-sm text-muted-foreground">
          {label}
          verifies and installs the bundle, then starts it once execution cleanup and plugin streams finish.
        </p>
      </div>
      <Button {disabled} onclick={onPick} size="sm" variant="outline">Choose File</Button>
    </div>
  </fieldset>
  {#if failure}
    <ErrorCard notificationContext="action" failure={failureOf(failure, "Operation Failed")} />
  {/if}
</div>
