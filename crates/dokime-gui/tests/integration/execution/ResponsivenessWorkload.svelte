<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Dialog } from "bits-ui";
  import type { ConnectedSession } from "../../../src/app/native-session/session";
  import PlotGroupGrid from "../../../src/components/document/PlotGroupGrid.svelte";
  import RecordTimeline from "../../../src/components/document/RecordTimeline.svelte";
  import Button from "../../../src/components/ui/Button.svelte";
  import DialogContent from "../../../src/components/ui/DialogContent.svelte";
  import type { DocumentSurfaceOptions } from "../../../src/features/cycle/document-view-model";
  import ExecutionSectionsView from "../../../src/features/cycle/ExecutionSectionsView.svelte";
  import ExecutionStepsCriteria from "../../../src/features/cycle/ExecutionStepsCriteria.svelte";
  import { createLiveDocumentSurface } from "../../../src/features/cycle/live-document.svelte";
  import { createPlotWindow } from "../../../src/features/cycle/plot-window-state";
  import type { DocumentViewPlot, StepExecutionRecord, TestExecution } from "../../../src/lib/types";
  import type { LivePlotStates } from "../../../src/plots/live-frames";
  import { fetchNativeRecordTail, openNativeRecordStream } from "./responsiveness-source";

  let {
    session,
    plots,
    options,
    initialFrames,
    synchronous,
  }: {
    session: ConnectedSession;
    plots: DocumentViewPlot[];
    options: DocumentSurfaceOptions;
    initialFrames: LivePlotStates;
    synchronous: boolean;
  } = $props();
  const time = "2026-11-01T00:00:00Z";
  const steps = [
    {
      record_type: "step",
      step_id: "capture",
      order: 0,
      label: "Capture",
      status: "running",
      metadata: { label: "Capture" },
      started_at: time,
      result: { details: {} },
    },
  ] as unknown as StepExecutionRecord[];
  const execution = { status: "running", started_at: time, finished_at: null, steps } as TestExecution;
  const plotWindow = createPlotWindow();
  function schedule(publish: () => void) {
    if (synchronous) {
      publish();
      return null;
    }
    return window.requestAnimationFrame(publish);
  }
  const surface = createLiveDocumentSurface(
    () => session,
    {
      get document() {
        return options.document;
      },
      execution,
      get live() {
        return options.live;
      },
      get snapshot() {
        return options.snapshot;
      },
      get scope() {
        return options.scope;
      },
      get parameters() {
        return options.parameters;
      },
      get noteElapsedStart() {
        return options.noteElapsedStart;
      },
      plotWindow,
    },
    {
      fetch: fetchNativeRecordTail,
      openStream: openNativeRecordStream,
      schedule,
    },
  );
  let frames = $state.raw<LivePlotStates>();
  let modal = $state(false);
  let ticks = $state(false);

  export function setStates(next: LivePlotStates) {
    frames = next;
  }
  export function setModal(open: boolean) {
    modal = open;
  }
  export function setTicks(enabled: boolean) {
    ticks = enabled;
  }
  export function currentView() {
    return surface.view;
  }
</script>

{#if ticks}
  <ExecutionSectionsView
    plan={null}
    row={{
      key: "execution",
      case: { caseId: "case", templateId: "template" } as never,
      execution,
      ordinal: 1,
      rerunOf: null,
      newest: true,
    }}
    live
    label="Measurement"
    titlePrefix=""
    onRename={null}
    started={execution}
    {surface}
    pane={{ kind: "document", staleReason: null }}
    {plotWindow}
  >
    {#snippet documentView()}
      <PlotGroupGrid documentPlots={plots} liveStates={frames ?? initialFrames} records={surface.view.records} title="Measurements" />
    {/snippet}
  </ExecutionSectionsView>
{:else}
  <ExecutionStepsCriteria {steps} criteria={[]} />
  <div style="height: 180px; overflow: auto"><RecordTimeline records={surface.view.records} /></div>
  <div class="grid grid-cols-2 gap-2">
    <PlotGroupGrid
      documentPlots={plots}
      liveStates={frames ?? initialFrames}
      records={surface.view.records}
      title="Measurements"
      showGroupHeaders={false}
    />
  </div>
  <Button id="transition-probe" style="position: fixed; right: 0; bottom: 0">Refresh</Button>
  <Dialog.Root bind:open={modal}>
    <DialogContent><Dialog.Title>Measurements</Dialog.Title></DialogContent>
  </Dialog.Root>
{/if}
