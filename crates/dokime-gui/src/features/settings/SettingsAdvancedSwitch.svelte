<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import Switch from "../../components/ui/Switch.svelte";
  import { storeState } from "../../stores/store-state";
  import { uiStore } from "../../stores/ui-store";

  const id = $props.id();
  const advanced = storeState(uiStore, (state) => state.showAdvancedSettings);
  const { setUiState } = uiStore.getState();
</script>

<div class="flex items-center gap-3 text-sm">
  <label for={id}>Show Advanced</label>
  <Switch {id} checked={advanced()} onCheckedChange={(showAdvancedSettings) => setUiState({ showAdvancedSettings })} />
</div>
