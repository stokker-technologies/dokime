<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { Check } from "lucide-svelte";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import PathPill from "../../components/PathPill.svelte";
  import Button from "../../components/ui/Button.svelte";
  import BackendsTable from "../../features/settings/BackendsTable.svelte";
  import { type Brand, defaultBrand } from "../../lib/brand";
  import { commands } from "../../lib/desktop/commands";
  import { DESKTOP_SESSION_STATUS_EVENT, type DesktopLaunch, type DesktopSessionSnapshot } from "../../lib/desktop/session";
  import { formatDecimal } from "../../lib/format";
  import { failureOf } from "../../lib/notification-details";
  import BrandMark from "../../shell/BrandMark.svelte";
  import { startupDiagnostics, storageRecoveryArchive, storageRecoveryAvailable } from "./errors";
  import { sessionState, type SessionOwner } from "./session-state.svelte";

  const RESET_PHASES = ["stopping", "archiving", "deleting", "reinstalling", "starting"] as const;
  let {
    brand = defaultBrand,
    launch,
    launchError,
    onSettings,
    owner,
  }: {
    brand?: Brand;
    launch: DesktopLaunch | null;
    launchError?: unknown;
    onSettings: () => void;
    owner: SessionOwner;
  } = $props();

  const latestConnection = sessionState(() => owner);
  const connection = $derived(latestConnection());
  const busy = $derived(connection.phase === "connecting");
  const reset = $derived(connection.phase === "connecting" && connection.reset !== false);
  const backingUp = $derived(connection.phase === "connecting" && connection.reset === "backup");
  const error = $derived(connection.phase === "failed" ? connection.error : (launchError ?? launch?.error));
  const diagnostics = $derived(startupDiagnostics(error));
  const archive = $derived(storageRecoveryArchive(error));
  const sentence = $derived(
    reset
      ? "Resetting local storage."
      : busy
        ? "Starting the backend."
        : !error && !launch
          ? "Reading this desktop's settings."
          : "Connect to a backend to plan and run cycles.",
  );

  let phase = $state<string | null>(null);
  let archivedBytes = $state<number | null>(null);
  let failedEndpoint = $state<string | null>(null);
  let localAttempt = $state(false);
  let selectedProfile = $state<string | null>(null);
  let statusRevision = -1;
  const recoverable = $derived(storageRecoveryAvailable(error, localAttempt, launch?.localBackendAvailable ?? false));
  const phases = $derived(
    reset
      ? RESET_PHASES.filter((step) => backingUp || step !== "archiving")
      : phase && phase !== "starting"
        ? ["starting", phase]
        : ["starting"],
  );
  const phaseIndex = $derived(
    reset && phase === null ? 0 : reset && phase && !phases.includes(phase) ? phases.length - 1 : phase ? phases.indexOf(phase) : -1,
  );

  $effect(() => {
    if (!busy) {
      phase = null;
      return;
    }
    let active = true;
    let unlisten: (() => void) | undefined;
    function applyStatus(status: DesktopSessionSnapshot) {
      if (!active || status.statusRevision < statusRevision) {
        return;
      }
      statusRevision = status.statusRevision;
      phase = status.phase;
      archivedBytes = status.archivedBytes ?? null;
    }

    void listen<DesktopSessionSnapshot>(DESKTOP_SESSION_STATUS_EVENT, ({ payload }) => applyStatus(payload))
      .then(async (dispose) => {
        if (!active) {
          dispose();
          return;
        }
        unlisten = dispose;
        const status = await commands.desktopSessionStatus();
        applyStatus(status);
      })
      .catch(() => {});
    return () => {
      active = false;
      unlisten?.();
    };
  });

  $effect(() => {
    failedEndpoint = null;
    if (!error || diagnostics) {
      return;
    }
    let active = true;
    void commands
      .desktopSessionStatus()
      .then((status) => {
        if (active) {
          failedEndpoint = status.connection.endpoint;
        }
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  });

  function startLocal() {
    localAttempt = true;
    void owner.select({ kind: "owned_local" });
  }

  function retry() {
    if (localAttempt || diagnostics) {
      void owner.select({ kind: "owned_local" });
      return;
    }
    if (selectedProfile) {
      void owner.select({ kind: "profile", identifier: selectedProfile });
      return;
    }
    if (launch?.target) {
      void owner.select({
        kind: "attach",
        request: { target: launch.target, expectedBackendId: launch.expectedBackendId },
      });
    } else if (launch?.localBackendAvailable) {
      void owner.select({ kind: "owned_local" });
    }
  }

  function phaseLabel(phase: string): string {
    switch (phase) {
      case "storage":
        return "Opening Storage";
      case "plugins":
        return "Loading Plugins";
      case "requests":
        return "Accepting Requests";
      case "sidecars":
        return "Starting Sidecars";
      case "stopping":
        return "Stopping Backend";
      case "archiving":
        return "Backing Up Local Storage";
      case "deleting":
        return "Deleting Local Storage";
      case "starting":
        return "Starting Backend";
      case "reinstalling":
        return "Reinstalling Plugins";
      default:
        return "Starting Backend";
    }
  }
</script>

<main class="flex min-h-screen items-center justify-center px-4 py-12 sm:px-6">
  <div class="grid w-full max-w-140 min-w-0 gap-6 text-center">
    <header class="grid justify-items-center gap-4">
      <BrandMark {brand} class="connection-brand brand-column" heading />
      <p class="text-sm text-muted-foreground" role={busy ? "status" : undefined}>{sentence}</p>
    </header>

    {#if busy}
      <ol aria-label={reset ? "Reset Steps" : "Startup Phases"} class="grid w-full gap-2 text-left">
        {#each phases as name, position (name)}
          {@const active = position === phaseIndex}
          {@const done = position < phaseIndex}
          <li
            class="grid grid-cols-[--spacing(5)_minmax(0,1fr)] items-center gap-3 text-sm"
            data-state={done ? "done" : active ? "active" : "waiting"}
          >
            <span class="grid size-5 place-items-center">
              {#if done}
                <Check aria-hidden="true" class="text-primary" size={14} />
              {:else}
                <span aria-hidden="true" class={["size-2 rounded-full", active ? "bg-primary" : "bg-border"]}></span>
              {/if}
            </span>
            <span class={active ? "font-medium text-foreground" : "text-muted-foreground"}>
              {phaseLabel(name)}
              {#if name === "archiving" && archivedBytes !== null}
                <span class="ml-2 font-mono text-xs text-muted-foreground tabular-nums">{formatDecimal(archivedBytes)} B</span>
              {/if}
            </span>
          </li>
        {/each}
      </ol>
      <div class="grid w-full gap-3">
        <Button class="w-full" disabled={reset} onclick={() => void owner.disconnect()} variant="outline">Cancel</Button>
      </div>
    {/if}

    <div class={busy ? "hidden" : "grid gap-6"}>
      {#if !launch && !error}
        <p role="status">Loading desktop settings…</p>
      {:else}
        {#if launch}
          <div class="grid w-full gap-2">
            <Button class="w-full" size="lg" disabled={!launch.localBackendAvailable} onclick={startLocal}>Start Local Backend</Button>
            <p class="text-xs text-muted-foreground">
              {launch.localBackendAvailable
                ? "This desktop owns the backend and closes it on exit."
                : `No ${brand.label} backend was found beside this application. Reinstall ${brand.label} with the desktop installer, or attach to a backend below.`}
            </p>
          </div>
        {/if}
        <div class="flex items-center gap-3 text-xs text-muted-foreground">
          <span class="h-px flex-1 bg-border"></span>
          <span>Or Connect to a Saved Backend</span>
          <span class="h-px flex-1 bg-border"></span>
        </div>
        <BackendsTable
          {owner}
          onConnect={(profile) => {
            localAttempt = false;
            selectedProfile = profile;
          }}
        />
      {/if}
    </div>
    {#if !busy && error}
      {@const endpoint = failedEndpoint ?? (launch?.target?.kind === "local" ? launch.target.endpoint : null)}
      <div class="grid gap-3 text-left text-sm">
        <ErrorCard
          notificationContext={selectedProfile ? "action" : "background"}
          failure={failureOf(error, diagnostics ? "Backend Could Not Start" : "Backend Connection Failed")}
          onRetry={retry}
        />
        {#if diagnostics?.log}
          <PathPill value={diagnostics.log} />
        {:else if endpoint}
          <p class="break-all font-mono text-xs">{endpoint}</p>
        {/if}
      </div>
    {/if}
    {#if !busy && archive}
      <PathPill value={archive} />
    {/if}
    {#if !busy && recoverable}
      <div class="grid gap-3">
        <Button
          variant="destructive"
          disabled
          title="The Host Must Preserve Plugins, Settings, Identity and Secrets Before Recovery Is Available"
          data-action="backup-reset"
        >
          Back Up and Reset
        </Button>
        <p class="text-sm text-muted-foreground">
          Recovery is unavailable until the host can preserve plugins, settings, identity and secrets. Preserve this folder and select a
          fresh storage root.
        </p>
      </div>
    {/if}
    <footer>
      <Button variant="link" disabled={busy} onclick={onSettings}>Settings</Button>
    </footer>
  </div>
</main>
