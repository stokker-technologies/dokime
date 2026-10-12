/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

/* === Local Imports === */

import { Slider } from "@/components/ui/slider";

import type { PlotCardSpec } from "@/plots/types";

/* === Types === */

type ChannelBand = {
  label: string;
  channels: number[];
};

type OccupancyReading = {
  channel: number;
  timeKey: string;
  timeValue: unknown;
  count: number;
};

type OccupancyScan = {
  key: string;
  timeValue: unknown;
  label: string;
  counts: Map<number, number>;
  occupiedChannels: number;
  peakCount: number;
  totalNetworks: number;
};

/* === Constants === */

const TWO_FOUR_GHZ_CHANNELS = Array.from({ length: 14 }, (_, index) => index + 1);
const FIVE_GHZ_CHANNELS = [
  32, 36, 40, 44, 48, 52, 56, 60, 64, 68, 96, 100, 104, 108, 112, 116, 120, 124, 128, 132, 136, 140,
  144, 149, 153, 157, 161, 165, 169, 173, 177,
];
const SIX_GHZ_CHANNELS = Array.from({ length: 59 }, (_, index) => 1_001 + index * 4);

/* === Components === */

export function WifiChannelOccupancyPlot({ plot }: { plot: PlotCardSpec }) {
  const readings = useMemo(() => occupancyReadings(plot), [plot]);
  const activeChannel = finiteNumber(
    plot.context?.active_wifi_channel ?? plot.context?.activeWifiChannel,
  );
  const activeBand = activeBandLabel(
    plot.context?.active_wifi_band ?? plot.context?.activeWifiBand,
    activeChannel,
  );
  const sampleKeys = useMemo(() => sampleTimeKeys(readings), [readings]);
  const scans = useMemo(() => occupancyScans(readings, sampleKeys), [readings, sampleKeys]);
  const latestIndex = Math.max(0, scans.length - 1);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [followLatest, setFollowLatest] = useState(true);

  useEffect(() => {
    setSelectedIndex((current) => (followLatest ? latestIndex : clamp(current, 0, latestIndex)));
  }, [followLatest, latestIndex]);

  const selectedScan = scans[selectedIndex] ?? null;
  const reportedChannels = [...new Set(readings.map((reading) => reading.channel))].sort(
    (left, right) => left - right,
  );
  const occupiedWindowChannels = [
    ...new Set(readings.filter((reading) => reading.count > 0).map((reading) => reading.channel)),
  ].sort((left, right) => left - right);
  const bands = channelBands(activeChannel, reportedChannels);
  const onlyActiveChannelReported =
    activeChannel !== null &&
    reportedChannels.length === 1 &&
    reportedChannels[0] === activeChannel;
  const activeLabel =
    activeChannel === null
      ? activeBand === null
        ? "No Active Channel"
        : `Active ${activeBand}`
      : `Active ${activeBand ?? channelBandLabel(activeChannel) ?? "WiFi"} Ch ${channelDisplay(activeChannel)}`;
  const summaryLabel = selectedScan
    ? `${scans.length} scans in window, selected ${selectedScan.label}, ${selectedScan.occupiedChannels} occupied channels, ${selectedScan.totalNetworks} networks`
    : (plot.stateDetail ?? "No nearby WiFi channel occupancy samples are available.");

  return (
    <article
      className="plot-card grid min-w-0 max-w-full overflow-hidden gap-3 rounded-md bg-muted/20 p-3"
      data-testid="wifi-channel-occupancy-plot"
    >
      <header className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h4 className="truncate text-sm font-semibold">{plot.title}</h4>
          <p className="text-xs text-muted-foreground">{summaryLabel}</p>
        </div>
        <div className="flex shrink-0 flex-wrap justify-end gap-2">
          {onlyActiveChannelReported ? (
            <span className="rounded-md border border-warning/40 bg-warning/10 px-2 py-1 text-xs font-semibold text-warning">
              Only Active Channel Reported
            </span>
          ) : null}
          <span className="rounded-md border bg-background px-2 py-1 text-xs font-semibold tabular-nums">
            {activeLabel}
          </span>
        </div>
      </header>
      <div className="grid gap-3">
        {bands.map((band) => (
          <BandHeatmap
            activeBand={activeBand}
            activeChannel={activeChannel}
            band={band}
            observedChannels={occupiedWindowChannels}
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
      {!readings.length ? (
        <p className="text-xs text-muted-foreground">
          {plot.stateDetail ?? "No nearby WiFi channel occupancy samples are available."}
        </p>
      ) : null}
    </article>
  );
}

/* === Private Components === */

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
  observedChannels: number[];
  scan: OccupancyScan | null;
}) {
  const isActiveBand = activeBand === band.label;
  const columns = `4.75rem repeat(${band.channels.length}, minmax(1.45rem, 1fr))`;
  const bandObservedChannels = observedChannels.filter((channel) =>
    band.channels.includes(channel),
  ).length;
  const bandCounts = band.channels.map((channel) => scan?.counts.get(channel) ?? 0);
  const occupiedChannels = bandCounts.filter((count) => count > 0).length;
  const totalNetworks = bandCounts.reduce((total, count) => total + count, 0);
  const peakCount = Math.max(0, ...bandCounts);

  return (
    <section
      className={`grid min-w-0 max-w-full gap-2 overflow-hidden rounded-md border p-2.5 ${
        isActiveBand ? "border-primary/50 bg-primary/5" : "border-transparent bg-background/60"
      }`}
    >
      <div className="flex min-w-0 items-center justify-between gap-3">
        <strong className="text-sm">{band.label}</strong>
        <span className="text-right text-xs text-muted-foreground">
          {occupiedChannels}/{band.channels.length} occupied now · {totalNetworks} networks
          {bandObservedChannels !== occupiedChannels
            ? ` · ${bandObservedChannels} seen in window`
            : ""}
        </span>
      </div>
      <div className="overflow-x-auto pb-1">
        <div className="grid min-w-[640px] gap-1" style={{ gridTemplateColumns: columns }}>
          <span />
          {band.channels.map((channel) => (
            <span
              className={`text-center text-[0.62rem] tabular-nums ${
                band.label === activeBand && channel === activeChannel
                  ? "font-bold text-primary"
                  : "text-muted-foreground"
              }`}
              key={channel}
            >
              {channelDisplay(channel)}
            </span>
          ))}
          <HeatmapRow
            activeBand={activeBand}
            activeChannel={activeChannel}
            band={band}
            peakCount={peakCount}
            scan={scan}
          />
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
      <span className="truncate text-right text-[0.65rem] text-muted-foreground">Occupancy</span>
      {band.channels.map((channel) => {
        const count = scan?.counts.get(channel) ?? 0;
        const isActiveChannel = band.label === activeBand && channel === activeChannel;
        return (
          <span
            aria-label={occupancyLabel(band.label, channel, count)}
            className={`h-6 min-w-6 rounded-lg ring-1 ${
              isActiveChannel ? "ring-2 ring-primary" : "ring-transparent"
            }`}
            key={channel}
            role="img"
            style={{
              backgroundColor: occupancyColor(count, peakCount),
              color: occupancyTextColor(count, peakCount),
            }}
            title={occupancyTitle(channel, count, scan)}
          >
            {count > 0 ? (
              <span className="grid h-full place-items-center text-[0.62rem] font-semibold leading-none tabular-nums">
                {count}
              </span>
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
    <div className="grid gap-1.5 border-t pt-2.5">
      <div className="flex items-center justify-between gap-3 text-xs text-muted-foreground">
        <span className="font-medium text-foreground">
          {followLatest ? "Latest Scan" : "Selected Scan"} {clampedSelectedIndex + 1}/{scans.length}
        </span>
        <span className="tabular-nums">
          {scan ? `${scan.label} · ${scan.totalNetworks} networks` : "No scan"}
        </span>
      </div>
      <Slider
        aria-label="WiFi occupancy scan"
        aria-valuetext={scan ? `${scan.label}, ${scan.totalNetworks} networks` : "No scan"}
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
      <div className="flex items-center justify-between gap-3 text-[0.65rem] text-muted-foreground">
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

/* === Private Functions === */

function occupancyReadings(plot: PlotCardSpec): OccupancyReading[] {
  return plot.series.flatMap((series) => {
    const channel = channelFromSeries(series.label, series.key);
    if (channel === null) {
      return [];
    }
    return series.points
      .map((point) => ({
        channel,
        timeKey: scanKey(point.x),
        timeValue: point.x,
        count: Math.max(0, Math.round(Number(point.y ?? 0))),
      }))
      .filter((reading) => Number.isFinite(reading.count));
  });
}

function sampleTimeKeys(readings: OccupancyReading[]): string[] {
  const rows = new Map<string, unknown>();
  readings.forEach((reading) => {
    rows.set(reading.timeKey, reading.timeValue);
  });
  return [...rows.entries()]
    .sort((left, right) => comparableTime(left[1]) - comparableTime(right[1]))
    .map(([key]) => key);
}

function occupancyScans(readings: OccupancyReading[], sampleKeys: string[]): OccupancyScan[] {
  const timeValues = new Map<string, unknown>();
  const countsByTime = new Map<string, Map<number, number>>();
  readings.forEach((reading) => {
    timeValues.set(reading.timeKey, reading.timeValue);
    let counts = countsByTime.get(reading.timeKey);
    if (!counts) {
      counts = new Map();
      countsByTime.set(reading.timeKey, counts);
    }
    counts.set(reading.channel, Math.max(counts.get(reading.channel) ?? 0, reading.count));
  });

  return sampleKeys.map((key) => {
    const counts = countsByTime.get(key) ?? new Map<number, number>();
    const values = [...counts.values()];
    const occupiedChannels = values.filter((count) => count > 0).length;
    const totalNetworks = values.reduce((total, count) => total + count, 0);
    const timeValue = timeValues.get(key);

    return {
      key,
      timeValue,
      label: scanTimeLabel(timeValue),
      counts,
      occupiedChannels,
      peakCount: Math.max(0, ...values),
      totalNetworks,
    };
  });
}

function channelFromSeries(label: string, key: string): number | null {
  const keyParts = key.split(":");
  return finiteNumber(label) ?? finiteNumber(keyParts[keyParts.length - 1]);
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
  const hasSixGhz = [activeChannel, ...observedChannels].some(
    (channel) => channel !== null && isEncodedSixGhzChannel(channel),
  );
  if (hasSixGhz) {
    bands.push({
      label: "6 GHz",
      channels: includeBandChannels(SIX_GHZ_CHANNELS, activeChannel, observedChannels),
    });
  }
  return bands;
}

function includeBandChannels(
  channels: number[],
  activeChannel: number | null,
  observedChannels: number[],
): number[] {
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
  return (
    channel >= 1_000 && displayChannel >= 1 && displayChannel <= 233 && displayChannel % 4 === 1
  );
}

function channelDisplay(channel: number): string {
  return isEncodedSixGhzChannel(channel) ? String(channel - 1_000) : String(channel);
}

function finiteNumber(value: unknown): number | null {
  const number = typeof value === "number" ? value : Number(value);
  return Number.isFinite(number) ? number : null;
}

function comparableTime(value: unknown): number {
  if (typeof value === "number" && Number.isFinite(value)) {
    return value;
  }
  if (typeof value === "string") {
    const parsed = Date.parse(value);
    return Number.isFinite(parsed) ? parsed : Number(value) || 0;
  }
  return 0;
}

function scanKey(value: unknown): string {
  if (typeof value === "string") {
    const parsed = Date.parse(value);
    if (Number.isFinite(parsed)) {
      return `time:${Math.floor(parsed / 1000)}`;
    }
  }
  if (typeof value === "number" && Number.isFinite(value)) {
    return `index:${value}`;
  }
  return String(value);
}

function scanTimeLabel(value: unknown): string {
  if (typeof value === "string") {
    const parsed = Date.parse(value);
    if (Number.isFinite(parsed)) {
      return new Date(parsed).toLocaleTimeString([], {
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
      });
    }
    return value;
  }
  if (typeof value === "number" && Number.isFinite(value)) {
    return Number.isInteger(value) ? String(value) : value.toFixed(2);
  }
  return "Unknown time";
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
  return peakCount > 0 && count / peakCount >= 0.65
    ? "var(--primary-foreground)"
    : "var(--foreground)";
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
