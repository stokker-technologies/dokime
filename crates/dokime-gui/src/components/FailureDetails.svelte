<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { failureText, type NotificationFailure } from "../lib/notification-details";

  let { failure }: { failure: NotificationFailure } = $props();
</script>

<details>
  <summary class="cursor-pointer rounded text-sm font-medium outline-none focus-visible:ring-2 focus-visible:ring-ring">
    Technical Details
  </summary>
  <pre class="mt-2 max-h-80 overflow-auto whitespace-pre-wrap break-words text-xs">{failureText(failure)}</pre>
</details>
