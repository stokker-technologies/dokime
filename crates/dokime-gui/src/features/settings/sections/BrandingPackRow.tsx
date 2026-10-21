/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Branding Pack Row Of The Brand Card
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

// Local Imports
import { Button } from "@/components/ui/button";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { type BrandingPackPresentation, brandingPackPresentation } from "@/features/settings/sections/branding-pack";
import type { BrandingStatus } from "@/lib/desktop/branding";
import { commands } from "@/lib/desktop/commands";

/* === Re-Exports === */

/* === Constants === */

const STATUS_KEY = ["settings", "branding-pack"] as const;

/* === Types === */

type BrandingPackViewProps = {
  presentation: BrandingPackPresentation;
  /** A refused archive, shown as the field error until the next choice. */
  choiceError: string | null;
  busy: boolean;
  choose: () => void;
  remove: () => void;
  restart: () => void;
};

/* === Components === */

/** The desktop shell's branding pack: chosen here, applied on the next start. */
export function BrandingPackRow() {
  const client = useQueryClient();
  const status = useQuery({ queryKey: STATUS_KEY, queryFn: commands.desktopBrandingStatus, retry: false });
  const publish = (next: BrandingStatus | null) => {
    if (next) client.setQueryData(STATUS_KEY, next);
  };
  const choose = useMutation({ mutationFn: commands.desktopBrandingChoose, onSuccess: publish });
  const remove = useMutation({ mutationFn: commands.desktopBrandingRemove, onSuccess: publish });
  const restart = useMutation({ mutationFn: commands.desktopRestart });
  if (!status.data) {
    return (
      <SettingsRow label="Branding Pack" error={status.error ? message(status.error) : null}>
        <span className="text-sm text-muted-foreground" role="status">
          {status.error ? "Unavailable" : "Reading the branding pack…"}
        </span>
      </SettingsRow>
    );
  }
  return (
    <BrandingPackView
      presentation={brandingPackPresentation(status.data)}
      choiceError={
        choose.error ? message(choose.error) : remove.error ? message(remove.error) : restart.error ? message(restart.error) : null
      }
      busy={choose.isPending || remove.isPending || restart.isPending}
      choose={() => choose.mutate()}
      remove={() => remove.mutate()}
      restart={() => restart.mutate()}
    />
  );
}

export function BrandingPackView({ presentation, choiceError, busy, choose, remove, restart }: BrandingPackViewProps) {
  const description = [presentation.detail, presentation.ignored].filter(Boolean).join(" ");
  return (
    <SettingsRow label="Branding Pack" description={description} error={choiceError ?? presentation.error}>
      <span className="min-w-0 text-sm font-medium" data-state={presentation.state}>
        {presentation.value}
      </span>
      <Button disabled={busy} onClick={choose} size="sm" variant="outline">
        Choose File…
      </Button>
      {presentation.restart ? (
        <Button disabled={busy} onClick={restart} size="sm">
          Restart
        </Button>
      ) : null}
      {presentation.remove ? (
        <Button disabled={busy} onClick={remove} size="sm" variant="ghost">
          Remove
        </Button>
      ) : null}
    </SettingsRow>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */

function message(error: unknown): string {
  return error instanceof Error ? error.message : typeof error === "string" ? error : "The branding pack could not be changed.";
}
