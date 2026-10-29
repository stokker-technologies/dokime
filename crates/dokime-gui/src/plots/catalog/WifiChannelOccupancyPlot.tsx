/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { Slider } from "@/components/ui/slider";

import type { PlotAnalysis, WifiOccupancyScan } from "@/lib/types";
import type { PlotCardSpec } from "@/plots/types";

type ChannelBand = {
  label: string;
  channels: number[];
};

type OccupancyAnalysis = Extract<PlotAnalysis, { kind: "wifi_occupancy" }>;

/** A backend scan with its display label and a channel lookup. */
type OccupancyScan = WifiOccupancyScan & {
  label: string;
  counts: Map<number, number>;
};

const TWO_FOUR_GHZ_CHANNELS = Array.from({ length: 14 }, (_, index) => index + 1);
const FIVE_GHZ_CHANNELS = [
  32, 36, 40, 44, 48, 52, 56, 60, 64, 68, 96, 100, 104, 108, 112, 116, 120, 124, 128, 132, 136, 140, 144, 149, 153, 157, 161, 165, 169, 173,
  177,
];
const SIX_GHZ_CHANNELS = Array.from({ length: 59 }, (_, index) => 1_001 + index * 4);

export function WifiChannelOccupancyPlot({ plot }: { plot: PlotCardSpec }) {
  const analysis = plot.analysis?.kind === "wifi_occupancy" ? plot.analysis : null;
  const activeChannel = finiteNumber(plot.context?.active_wifi_channel ?? plot.context?.activeWifiChannel);
  const activeBand = activeBandLabel(plot.context?.active_wifi_band ?? plot.context?.activeWifiBand, activeChannel);
  const scans = useMemo(() => occupancyScans(analysis), [analysis]);
  const latestIndex = Math.max(0, scans.length - 1);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [followLatest, setFollowLatest] = useState(true);

  useEffect(() => {
    setSelectedIndex((current) => (followLatest ? latestIndex : clamp(current, 0, latestIndex)));
  }, [followLatest, latestIndex]);

  const selectedScan = scans[selectedIndex] ?? null;
  const reportedChannels = analysis?.channels ?? [];
  const bands = channelBands(activeChannel, reportedChannels);
  const onlyActiveChannelReported = activeChannel !== null && reportedChannels.length === 1 && reportedChannels[0] === activeChannel;
  const activeLabel =
    activeChannel === null
      ? activeBand === null
        ? "No Active Channel"
        : `Active ${activeBand}`
      : `Active ${activeBand ?? channelBandLabel(activeChannel) ?? "WiFi"} Ch ${channelDisplay(activeChannel)}`;
  const summaryLabel = selectedScan
    ? `${scans.length} scans in window, selected ${selectedScan.label}, ${selectedScan.occupied_channels} occupied channels, ${selectedScan.total_networks} networks`
    : (plot.stateDetail ?? "No nearby WiFi channel occupancy samples are available.");

  return (
    <article
      className="plot-card grid min-w-0 max-w-full overflow-hidden gap-3 rounded-lg bg-muted/20 p-3"
      data-testid="wifi-channel-occupancy-plot"
    >
      <header className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h4 className="truncate text-sm font-semibold">{plot.title}</h4>
          <p className="text-xs text-muted-foreground">{summaryLabel}</p>
        </div>
        <div className="flex shrink-0 flex-wrap justify-end gap-2">
          {onlyActiveChannelReported ? (
            <span className="rounded-lg border border-warning/40 bg-warning/10 px-2 py-1 text-xs font-semibold text-warning">
              Only Active Channel Reported
            </span>
          ) : null}
          <span className="rounded-lg border bg-background px-2 py-1 text-xs font-semibold tabular-nums">{activeLabel}</span>
        </div>
      </header>
      <div className="grid gap-3">
        {bands.map((band) => (
          <BandHeatmap
            activeBand={activeBand}
            activeChannel={activeChannel}
            band={band}
            observedChannels={analysis?.bands.find((entry) => entry.band === band.label)?.occupied_channels ?? 0}
            key={band.label}
            scan={selectedScan}
          />
        ))}
      </div>
      {scans.length > 1 ? (
        <ScanSlider
          followLatest={followLatest}
          latestIndex={latestIndex}
          scan={selectedScan}
          scans={scans}
          selectedIndex={selectedIndex}
          onCommit={(index) => {
            const next = clamp(index, 0, latestIndex);
            setSelectedIndex(next);
            setFollowLatest(next === latestIndex);
          }}
          onLatest={() => {
            setSelectedIndex(latestIndex);
            setFollowLatest(true);
          }}
          onPreview={(index) => {
            setSelectedIndex(clamp(index, 0, latestIndex));
          }}
        />
      ) : null}
      {!scans.length ? (
        <p className="text-xs text-muted-foreground">{plot.stateDetail ?? "No nearby WiFi channel occupancy samples are available."}</p>
      ) : null}
    </article>
  );
}

function BandHeatmap({
  activeBand,
  activeChannel,
  band,
  observedChannels,
  scan,
}: {
  activeBand: string | null;
  activeChannel: number | null;
  band: ChannelBand;
  observedChannels: number;
  scan: OccupancyScan | null;
}) {
  const isActiveBand = activeBand === band.label;
  const columns = `4.75rem repeat(${band.channels.length}, minmax(1.45rem, 1fr))`;
  const bandScan = scan?.bands.find((entry) => entry.band === band.label);
  const occupiedChannels = bandScan?.occupied_channels ?? 0;
  const totalNetworks = bandScan?.total_networks ?? 0;
  const peakCount = bandScan?.peak_count ?? 0;

  return (
    <section
      className={`grid min-w-0 max-w-full gap-2 overflow-hidden rounded-lg border p-3 ${
        isActiveBand ? "border-primary/50 bg-primary/5" : "border-transparent bg-background/60"
      }`}
    >
      <div className="flex min-w-0 items-center justify-between gap-3">
        <strong className="text-sm">{band.label}</strong>
        <span className="text-right text-xs text-muted-foreground">
          {occupiedChannels}/{band.channels.length} Occupied Now · {totalNetworks} Networks
          {observedChannels !== occupiedChannels ? ` · ${observedChannels} seen in window` : ""}
        </span>
      </div>
      <div className="overflow-x-auto pb-1">
        <div className="grid min-w-160 gap-1" style={{ gridTemplateColumns: columns }}>
          <span />
          {band.channels.map((channel) => (
            <span
              className={`text-center text-xs tabular-nums ${
                band.label === activeBand && channel === activeChannel ? "font-bold text-primary" : "text-muted-foreground"
              }`}
              key={channel}
            >
              {channelDisplay(channel)}
            </span>
          ))}
          <HeatmapRow activeBand={activeBand} activeChannel={activeChannel} band={band} peakCount={peakCount} scan={scan} />
        </div>
      </div>
    </section>
  );
}

function HeatmapRow({
  activeBand,
  activeChannel,
  band,
  peakCount,
  scan,
}: {
  activeBand: string | null;
  activeChannel: number | null;
  band: ChannelBand;
  peakCount: number;
  scan: OccupancyScan | null;
}) {
  return (
    <>
      <span className="truncate text-right text-xs text-muted-foreground">Occupancy</span>
      {band.channels.map((channel) => {
        const count = scan?.counts.get(channel) ?? 0;
        const isActiveChannel = band.label === activeBand && channel === activeChannel;
        return (
          <span
            aria-label={occupancyLabel(band.label, channel, count)}
            className={`h-6 min-w-6 rounded-lg ring-1 ${isActiveChannel ? "ring-2 ring-primary" : "ring-transparent"}`}
            key={channel}
            role="img"
            style={{
              backgroundColor: occupancyColor(count, peakCount),
              color: occupancyTextColor(count, peakCount),
            }}
            title={occupancyTitle(channel, count, scan)}
          >
            {count > 0 ? (
              <span className="grid h-full place-items-center text-xs font-semibold leading-none tabular-nums">{count}</span>
            ) : null}
          </span>
        );
      })}
    </>
  );
}

function ScanSlider({
  followLatest,
  latestIndex,
  scan,
  scans,
  selectedIndex,
  onCommit,
  onLatest,
  onPreview,
}: {
  followLatest: boolean;
  latestIndex: number;
  scan: OccupancyScan | null;
  scans: OccupancyScan[];
  selectedIndex: number;
  onCommit: (index: number) => void;
  onLatest: () => void;
  onPreview: (index: number) => void;
}) {
  const interactingRef = useRef(false);
  const clampedSelectedIndex = clamp(selectedIndex, 0, latestIndex);
  const sliderValue = useMemo(() => [clampedSelectedIndex], [clampedSelectedIndex]);
  const handleValueChange = useCallback(
    (value: number[]) => {
      if (!interactingRef.current) {
        return;
      }
      onPreview(Math.round(value[0] ?? latestIndex));
    },
    [latestIndex, onPreview],
  );
  const handleValueCommit = useCallback(
    (value: number[]) => {
      interactingRef.current = false;
      onCommit(Math.round(value[0] ?? latestIndex));
    },
    [latestIndex, onCommit],
  );

  return (
    <div className="grid gap-2 border-t pt-2">
      <div className="flex items-center justify-between gap-3 text-xs text-muted-foreground">
        <span className="font-medium text-foreground">
          {followLatest ? "Latest Scan" : "Selected Scan"} {clampedSelectedIndex + 1}/{scans.length}
        </span>
        <span className="tabular-nums">{scan ? `${scan.label} · ${scan.total_networks} networks` : "No scan"}</span>
      </div>
      <Slider
        aria-label="WiFi Occupancy Scan"
        aria-valuetext={scan ? `${scan.label}, ${scan.total_networks} networks` : "No scan"}
        max={latestIndex}
        min={0}
        step={1}
        value={sliderValue}
        onBlur={() => {
          interactingRef.current = false;
        }}
        onKeyDown={() => {
          interactingRef.current = true;
        }}
        onPointerDown={() => {
          interactingRef.current = true;
        }}
        onValueChange={handleValueChange}
        onValueCommit={handleValueCommit}
      />
      <div className="flex items-center justify-between gap-3 text-xs text-muted-foreground">
        <span className="truncate">{scans[0]?.label ?? ""}</span>
        <button
          className="text-xs font-semibold text-primary disabled:text-muted-foreground"
          disabled={followLatest}
          type="button"
          onClick={onLatest}
        >
          Latest
        </button>
        <span className="truncate text-right">{scans[latestIndex]?.label ?? ""}</span>
      </div>
    </div>
  );
}

function occupancyScans(analysis: OccupancyAnalysis | null): OccupancyScan[] {
  return (analysis?.scans ?? []).map((scan) => ({
    ...scan,
    label: scanTimeLabel(scan.time),
    counts: new Map(scan.channels.map((entry) => [entry.channel, entry.count])),
  }));
}

function channelBands(activeChannel: number | null, observedChannels: number[]): ChannelBand[] {
  const bands: ChannelBand[] = [
    {
      label: "2.4 GHz",
      channels: includeBandChannels(TWO_FOUR_GHZ_CHANNELS, activeChannel, observedChannels),
    },
    {
      label: "5 GHz",
      channels: includeBandChannels(FIVE_GHZ_CHANNELS, activeChannel, observedChannels),
    },
  ];
  const hasSixGhz = [activeChannel, ...observedChannels].some((channel) => channel !== null && isEncodedSixGhzChannel(channel));
  if (hasSixGhz) {
    bands.push({
      label: "6 GHz",
      channels: includeBandChannels(SIX_GHZ_CHANNELS, activeChannel, observedChannels),
    });
  }
  return bands;
}

function includeBandChannels(channels: number[], activeChannel: number | null, observedChannels: number[]): number[] {
  const bandLabel = channelBandLabel(channels[0] ?? 0);
  const extras = [activeChannel, ...observedChannels].filter(
    (channel): channel is number => channel !== null && channelBandLabel(channel) === bandLabel,
  );
  if (!extras.length) {
    return channels;
  }
  return [...new Set([...channels, ...extras])].sort((left, right) => left - right);
}

function activeBandLabel(value: unknown, activeChannel: number | null): string | null {
  const text = typeof value === "string" ? value.toLowerCase() : "";
  if (text.includes("2.4")) {
    return "2.4 GHz";
  }
  if (text.includes("5")) {
    return "5 GHz";
  }
  if (text.includes("6")) {
    return "6 GHz";
  }
  return activeChannel === null ? null : channelBandLabel(activeChannel);
}

function channelBandLabel(channel: number): string | null {
  if (channel >= 1 && channel <= 14) {
    return "2.4 GHz";
  }
  if (channel >= 32 && channel <= 177) {
    return "5 GHz";
  }
  if (isEncodedSixGhzChannel(channel)) {
    return "6 GHz";
  }
  return null;
}

function isEncodedSixGhzChannel(channel: number): boolean {
  const displayChannel = channel - 1_000;
  return channel >= 1_000 && displayChannel >= 1 && displayChannel <= 233 && displayChannel % 4 === 1;
}

function channelDisplay(channel: number): string {
  return isEncodedSixGhzChannel(channel) ? String(channel - 1_000) : String(channel);
}

function finiteNumber(value: unknown): number | null {
  const number = typeof value === "number" ? value : Number(value);
  return Number.isFinite(number) ? number : null;
}

function scanTimeLabel(value: string): string {
  const parsed = Date.parse(value);
  if (!Number.isFinite(parsed)) {
    return value;
  }
  return new Date(parsed).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

function occupancyLabel(band: string, channel: number, count: number): string {
  return `${band} channel ${channelDisplay(channel)}: ${count} network${count === 1 ? "" : "s"}`;
}

function occupancyTitle(channel: number, count: number, scan: OccupancyScan | null): string {
  const time = scan ? ` at ${scan.label}` : "";
  return `Ch ${channelDisplay(channel)}${time}: ${count} network${count === 1 ? "" : "s"}`;
}

function occupancyColor(count: number, peakCount: number): string {
  if (count <= 0) {
    return "color-mix(in srgb, var(--muted-foreground) 10%, transparent)";
  }
  const ratio = peakCount <= 0 ? 1 : Math.min(1, count / peakCount);
  const alpha = 0.22 + ratio * 0.68;
  return `color-mix(in srgb, var(--primary) ${(alpha * 100).toFixed(1)}%, transparent)`;
}

function occupancyTextColor(count: number, peakCount: number): string {
  if (count <= 0) {
    return "var(--foreground)";
  }
  return peakCount > 0 && count / peakCount >= 0.65 ? "var(--primary-foreground)" : "var(--foreground)";
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
