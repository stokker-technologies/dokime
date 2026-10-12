/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

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
  type LucideIcon,
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
} from "lucide-react";

/* === Local Imports === */

import { recordToneShellClass } from "@/lib/record-display";
import type { RecordDisplay } from "@/lib/types";
import { cn } from "@/lib/utils";

/* === Constants === */

const RECORD_ICONS: Record<RecordDisplay["icon"], LucideIcon> = {
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

/* === Public Functions === */

export function RecordGlyph({
  className,
  display,
}: {
  className?: string;
  display: RecordDisplay;
}) {
  const Icon = RECORD_ICONS[display.icon] ?? Clock3;

  return (
    <span
      aria-hidden="true"
      className={recordToneShellClass(
        display.tone,
        cn("inline-grid size-7 shrink-0 place-items-center rounded-md [&_svg]:size-4", className),
      )}
    >
      <Icon />
    </span>
  );
}
