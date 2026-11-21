/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Installed-plugin settings cards. */

/* === External Imports === */

/* === Local Imports === */

import { Badge } from "@dokime/plots/components/ui/badge";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import * as model from "@host/model";
import type { Wire } from "@dokime/plots/lib/rpc/canonical";

/* === Types === */

type PluginSettingsCardsProps = { cards: Wire<model.SettingsCard>[] };

/* === Components === */

export function PluginSettingsCards({ cards }: PluginSettingsCardsProps) {
  if (cards.length === 0) {
    return null;
  }

  return (
    <>
      {cards.map((card) => (
        <PluginSettingsCard card={card} key={`${card.pluginId}:${card.cardId}`} />
      ))}
    </>
  );
}

function PluginSettingsCard({ card }: { card: Wire<model.SettingsCard> }) {
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
      {(card.body?.blocks ?? []).map((block) => (
        <HostCardBlockView block={block} key={hostCardBlockKey(block)} />
      ))}
    </SettingsCard>
  );
}

function HostCardBlockView({ block }: { block: Wire<model.HostCardBlock> }) {
  const variant = block.kind?.kind;
  switch (variant?.case) {
    case "text":
      return <div className="px-4 py-2 text-sm text-muted-foreground">{variant.value}</div>;
    case "status": {
      const tone = toneMeta(variant.value.tone);
      return (
        <SettingsRow
          control={<Badge variant={tone.variant}>{tone.label}</Badge>}
          description={variant.value.detail}
          label={variant.value.label}
        />
      );
    }
    case "keyValueList":
      return (
        <>
          {(variant.value.items ?? []).map((item) => (
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
          {(variant.value.items ?? []).map((item) => (
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

/* === Private Functions === */

function stateMeta(state: model.SettingsCardState): {
  label: string;
  variant: "secondary" | "success" | "warning" | "destructive" | "outline";
} {
  switch (state) {
    case model.SettingsCardState.READY:
      return { label: "Ready", variant: "success" };
    case model.SettingsCardState.DISABLED:
      return { label: "Disabled", variant: "secondary" };
    case model.SettingsCardState.DISCONNECTED:
      return { label: "Disconnected", variant: "warning" };
    case model.SettingsCardState.STALE:
      return { label: "Stale", variant: "warning" };
    case model.SettingsCardState.ERROR:
      return { label: "Error", variant: "destructive" };
    case model.SettingsCardState.PERMISSION_DENIED:
      return { label: "Permission Denied", variant: "destructive" };
    default:
      return { label: "Unknown", variant: "outline" };
  }
}

function toneMeta(tone: model.HostCardTone): {
  label: string;
  variant: "secondary" | "success" | "warning" | "destructive" | "outline";
} {
  switch (tone) {
    case model.HostCardTone.SUCCESS:
      return { label: "OK", variant: "success" };
    case model.HostCardTone.WARNING:
      return { label: "Warning", variant: "warning" };
    case model.HostCardTone.DANGER:
      return { label: "Error", variant: "destructive" };
    case model.HostCardTone.INFO:
      return { label: "Info", variant: "outline" };
    default:
      return { label: "Status", variant: "secondary" };
  }
}

function hostCardBlockKey(block: Wire<model.HostCardBlock>): string {
  return JSON.stringify(block);
}
