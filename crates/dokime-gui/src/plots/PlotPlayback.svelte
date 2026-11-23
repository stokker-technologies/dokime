<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import Button from "../components/ui/Button.svelte";
  import SelectField from "../components/ui/SelectField.svelte";
  import Slider from "../components/ui/Slider.svelte";
  import { PLAYBACK_SPEEDS, type plotPlayback } from "./playback.svelte";

  let { title, playback }: { title: string; playback: Omit<ReturnType<typeof plotPlayback>, "data"> } = $props();
  const disabled = $derived(playback.instants.length < 2);
  const speeds = PLAYBACK_SPEEDS.map((speed) => ({ value: `${speed}`, label: `${speed}×` }));
  let interacting = false;

  function begin() {
    interacting = true;
  }

  function end() {
    interacting = false;
  }
</script>

<fieldset class="grid min-w-0 gap-2" aria-label={`${title} Playback`} data-playback-instant={playback.instant ?? ""}>
  <Slider
    aria-label={`${title} Instant`}
    aria-valuetext={playback.instant ?? "No Instants"}
    min={0}
    max={Math.max(1, playback.instants.length - 1)}
    step={1}
    value={[playback.index]}
    {disabled}
    onkeydown={begin}
    onpointerdown={begin}
    onfocusout={end}
    onValueChange={([next]) => {
      if (interacting) {
        playback.seek(next);
      }
    }}
    onValueCommit={([next]) => {
      end();
      playback.seek(next, true);
    }}
  />
  <div class="flex flex-wrap items-center gap-2">
    <Button variant="outline" size="xs" disabled={disabled || playback.playing} onclick={playback.play}>Play</Button>
    <Button variant="outline" size="xs" disabled={!playback.playing} onclick={playback.pause}>Pause</Button>
    <Button variant="outline" size="xs" {disabled} onclick={playback.replay}>Replay</Button>
    <Button variant="outline" size="xs" disabled={!playback.instants.length || playback.following} onclick={playback.latest}>Latest</Button>
    <SelectField
      label={`${title} Playback Speed`}
      size="sm"
      class="text-xs"
      value={`${playback.speed}`}
      options={speeds}
      {disabled}
      onChange={(next) => {
        playback.speed = Number(next);
      }}
    />
    <output class="min-w-0 truncate text-xs tabular-nums text-muted-foreground">{playback.instant ?? "No Instants"}</output>
  </div>
</fieldset>
