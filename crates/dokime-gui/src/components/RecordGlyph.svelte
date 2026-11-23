<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import {
    Activity,
    AlertTriangle,
    Archive,
    Bell,
    CheckCircle2,
    CircleDashed,
    ClipboardCheck,
    Clock3,
    Cpu,
    FileText,
    Flag,
    Gauge,
    HeartPulse,
    HelpCircle,
    MessageSquareText,
    Package,
    Plug,
    RadioTower,
    RotateCcw,
    ShieldAlert,
    Target,
    TimerReset,
    Wrench,
    Zap,
  } from "lucide-svelte";

  import { recordToneShellClass } from "@/lib/record-display";
  import type { RecordDisplay } from "@/lib/types";
  import { cn } from "@/lib/utils";

  const RECORD_ICONS: Record<RecordDisplay["icon"], typeof Clock3> = {
    activity: Activity,
    "alert-triangle": AlertTriangle,
    archive: Archive,
    bell: Bell,
    "check-circle": CheckCircle2,
    "circle-dashed": CircleDashed,
    "clipboard-check": ClipboardCheck,
    clock: Clock3,
    cpu: Cpu,
    "file-text": FileText,
    flag: Flag,
    gauge: Gauge,
    "heart-pulse": HeartPulse,
    "help-circle": HelpCircle,
    "message-square-text": MessageSquareText,
    package: Package,
    plug: Plug,
    "radio-tower": RadioTower,
    "rotate-ccw": RotateCcw,
    "shield-alert": ShieldAlert,
    target: Target,
    "timer-reset": TimerReset,
    wrench: Wrench,
    zap: Zap,
  };

  let { class: style, display }: { class?: string; display: RecordDisplay } = $props();
  const Icon = $derived(RECORD_ICONS[display.icon] ?? Clock3);
</script>

<span
  aria-hidden="true"
  class={recordToneShellClass(display.tone, cn("inline-grid size-7 shrink-0 place-items-center rounded-lg [&_svg]:size-4", style))}
>
  <Icon />
</span>
