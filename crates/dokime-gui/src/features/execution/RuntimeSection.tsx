/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { Activity, Clock3, Cpu, RadioTower } from "lucide-react";
import { type ReactNode, useEffect, useState } from "react";

/* === Local Imports === */

import { CollapsibleSection } from "@/components/CollapsibleSection";
import { Badge } from "@/components/ui/badge";
import { formatBytes, formatDuration } from "@/lib/format";
import { elapsedLabel } from "@/lib/time-format";
import type { DiagnosticsSnapshot, DocumentCardLayout, HubLiveState } from "@/lib/types";
import { useHubStore } from "@/stores/hub-store";

const GUI_ELAPSED_TICK_MS = 1_000;

/* === Types === */

type RuntimeSectionProps = {
  connection: "connecting" | "connected" | "reconnecting";
  diagnostics: DiagnosticsSnapshot | null;
  layout?: DocumentCardLayout;
  snapshot: HubLiveState | null;
};

/* === Components === */

export function RuntimeSection({ connection, diagnostics, layout, snapshot }: RuntimeSectionProps) {
  const now = useNow(GUI_ELAPSED_TICK_MS);
  const artifacts = snapshot?.installed_plugins ?? [];
  const readyArtifacts = artifacts.filter((artifact) => artifact.ready).length;
  const failedArtifacts = artifacts.filter((artifact) => artifact.status === "failed").length;
  const artifactSummary = artifacts.length ? `${readyArtifacts}/${artifacts.length} artifacts ready` : "Artifacts ready";
  const process = diagnostics?.server.process;
  const runStartedAt = snapshot?.run.started_at ?? null;
  const runFinishedAt = snapshot?.run.finished_at ?? null;
  const lastStreamFrameAt = useHubStore.getState().lastStreamFrameAt;

  return (
    <CollapsibleSection
      contentClassName="grid gap-4 p-3"
      defaultOpen={false}
      eyebrow="Runtime"
      icon={<Activity />}
      layout={layout}
      summary={`Health Clear · ${artifactSummary}`}
      title="Runtime Health"
    >
      <div className="grid gap-3 [grid-template-columns:repeat(auto-fit,minmax(min(100%,10rem),1fr))]">
        <RuntimeMetric
          icon={<Clock3 />}
          label="Run Elapsed"
          value={elapsedLabel(runStartedAt, now, runFinishedAt)}
          detail={
            lastStreamFrameAt !== null
              ? `Live frame ${formatDuration(Math.max(0, Math.floor((now - lastStreamFrameAt) / 1000)))} ago`
              : "No live frames yet"
          }
        />
        <RuntimeMetric
          icon={<RadioTower />}
          label="Connection"
          value={connection}
          detail={`Doctor ${diagnostics?.runtime.startup_doctor.status ?? "unknown"}`}
          tone={connection === "connected" ? "good" : "bad"}
        />
        <RuntimeMetric
          icon={<Cpu />}
          label="Process"
          value={`CPU ${formatPercentValue(process?.cpu_percent)}`}
          detail={`Mem ${formatBytes(process?.memory_bytes)}`}
        />
      </div>

      <section className="grid gap-2">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <strong className="text-sm">Artifact Artifacts</strong>
          <Badge variant={failedArtifacts ? "secondary" : "outline"}>
            {readyArtifacts}/{artifacts.length || 0} ready
          </Badge>
        </div>
        <div className="grid gap-2 sm:grid-cols-2">
          {artifacts.length ? (
            artifacts.map((artifact) => (
              <div className="grid gap-1 rounded-lg border bg-muted/20 p-3 text-sm" key={artifact.artifact_id}>
                <div className="flex min-w-0 items-center justify-between gap-2">
                  <strong className="truncate">{artifact.artifact_id}</strong>
                  <Badge variant={artifact.ready ? "outline" : "secondary"}>{artifact.status}</Badge>
                </div>
                <p className="text-xs text-muted-foreground">
                  {artifact.runtime} · {artifact.domain_registrations} domain registrations
                  {artifact.cache_hit ? " · cache hit" : ""}
                </p>
                {artifact.last_error ? <p className="text-xs text-destructive [overflow-wrap:anywhere]">{artifact.last_error}</p> : null}
              </div>
            ))
          ) : (
            <p className="text-sm text-muted-foreground">No artifact readiness details reported.</p>
          )}
        </div>
      </section>
    </CollapsibleSection>
  );
}

function RuntimeMetric({
  detail,
  icon,
  label,
  tone,
  value,
}: {
  detail: string;
  icon: ReactNode;
  label: string;
  tone?: "good" | "warn" | "bad";
  value: string;
}) {
  return (
    <div className="grid gap-2 rounded-lg border bg-muted/25 p-3">
      <div className="flex items-center justify-between gap-2 text-muted-foreground">
        <span className="text-xs font-bold uppercase">{label}</span>
        {icon}
      </div>
      <strong className={tone === "bad" ? "text-destructive" : tone === "warn" ? "text-warning" : tone === "good" ? "text-success" : ""}>
        {value}
      </strong>
      <span className="text-xs text-muted-foreground">{detail}</span>
    </div>
  );
}

function useNow(intervalMs: number): number {
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    const handle = window.setInterval(() => setNow(Date.now()), intervalMs);
    return () => window.clearInterval(handle);
  }, [intervalMs]);

  return now;
}

function formatPercentValue(value: number | null | undefined): string {
  if (value === null || value === undefined || !Number.isFinite(value)) {
    return "-";
  }
  return `${value.toFixed(1)}%`;
}
