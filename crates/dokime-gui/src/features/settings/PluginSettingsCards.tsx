/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Installed-plugin settings cards. */

import { Badge } from "@/components/ui/badge";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import type { HostCardBlock, HostCardTone, RuntimeSettingsCard, RuntimeSettingsCardState } from "@/features/settings/settings-contracts";

type PluginSettingsCardsProps = { cards: RuntimeSettingsCard[] };

export function PluginSettingsCards({ cards }: PluginSettingsCardsProps) {
  if (cards.length === 0) {
    return null;
  }

  return (
    <>
      {cards.map((card) => (
        <PluginSettingsCard card={card} key={`${card.plugin_id}:${card.card_id}`} />
      ))}
    </>
  );
}

function PluginSettingsCard({ card }: { card: RuntimeSettingsCard }) {
  const state = stateMeta(card.state);

  return (
    <SettingsCard
      description={card.description}
      footer={
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex min-w-0 flex-col gap-1">
            <Badge variant={state.variant}>{state.label}</Badge>
          </div>
        </div>
      }
      title={card.label}
    >
      {(card.body.blocks ?? []).map((block) => (
        <HostCardBlockView block={block} key={hostCardBlockKey(block)} />
      ))}
    </SettingsCard>
  );
}

function HostCardBlockView({ block }: { block: HostCardBlock }) {
  switch (block.type) {
    case "text":
      return <div className="px-4 py-2 text-sm text-muted-foreground">{block.text}</div>;
    case "status": {
      const tone = toneMeta(block.tone);
      return <SettingsRow control={<Badge variant={tone.variant}>{tone.label}</Badge>} description={block.detail} label={block.label} />;
    }
    case "key_value_list":
      return (
        <>
          {(block.items ?? []).map((item) => (
            <SettingsRow
              control={
                <span className="min-w-0 truncate text-sm text-muted-foreground" title={item.value}>
                  {item.value}
                </span>
              }
              description={item.detail}
              key={item.label}
              label={item.label}
            />
          ))}
        </>
      );
    case "metrics":
      return (
        <div className="grid gap-3 px-4 py-3 sm:grid-cols-2 lg:grid-cols-3">
          {(block.items ?? []).map((item) => (
            <div className="min-w-0 rounded-lg border bg-muted/20 px-3 py-2" key={item.label}>
              <div className="truncate text-xs text-muted-foreground">{item.label}</div>
              <div className="mt-1 flex items-baseline gap-1">
                <span className="truncate text-lg font-semibold">{item.value}</span>
                {item.unit ? <span className="text-xs text-muted-foreground">{item.unit}</span> : null}
              </div>
              {item.trend ? <div className="mt-1 truncate text-xs text-muted-foreground">{item.trend}</div> : null}
            </div>
          ))}
        </div>
      );
  }
}

function stateMeta(state: RuntimeSettingsCardState): {
  label: string;
  variant: "secondary" | "success" | "warning" | "destructive" | "outline";
} {
  switch (state) {
    case "ready":
      return { label: "Ready", variant: "success" };
    case "disabled":
      return { label: "Disabled", variant: "secondary" };
    case "disconnected":
      return { label: "Disconnected", variant: "warning" };
    case "stale":
      return { label: "Stale", variant: "warning" };
    case "error":
      return { label: "Error", variant: "destructive" };
    case "permission_denied":
      return { label: "Permission Denied", variant: "destructive" };
  }
}

function toneMeta(tone: HostCardTone): {
  label: string;
  variant: "secondary" | "success" | "warning" | "destructive" | "outline";
} {
  switch (tone) {
    case "success":
      return { label: "OK", variant: "success" };
    case "warning":
      return { label: "Warning", variant: "warning" };
    case "danger":
      return { label: "Error", variant: "destructive" };
    case "info":
      return { label: "Info", variant: "outline" };
    case "neutral":
      return { label: "Status", variant: "secondary" };
  }
}

function hostCardBlockKey(block: HostCardBlock): string {
  return `${block.type}:${JSON.stringify(block)}`;
}
