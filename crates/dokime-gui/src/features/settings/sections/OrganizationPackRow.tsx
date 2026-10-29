/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Organization Pack Row Of The Brand Card
 */

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { Button } from "@/components/ui/button";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { type OrganizationPackPresentation, organizationPackPresentation } from "@/features/settings/sections/organization-pack";
import { commands } from "@/lib/desktop/commands";
import type { OrganizationStatus } from "@/lib/desktop/organization";

const STATUS_KEY = ["settings", "organization-pack"] as const;

type OrganizationPackViewProps = {
  presentation: OrganizationPackPresentation;
  /** A refused archive, delivered through notifications without replacing the current pack. */
  choiceError: string | null;
  busy: boolean;
  choose: () => void;
  remove: () => void;
  restart: () => void;
};

/** The desktop shell's organization pack: chosen here, applied on the next start. */
export function OrganizationPackRow() {
  const client = useQueryClient();
  const status = useQuery({ queryKey: STATUS_KEY, queryFn: commands.desktopOrganizationStatus, retry: false });
  const publish = (next: OrganizationStatus | null) => {
    if (next) client.setQueryData(STATUS_KEY, next);
  };
  const choose = useMutation({ mutationFn: commands.desktopOrganizationChoose, onSuccess: publish });
  const remove = useMutation({ mutationFn: commands.desktopOrganizationRemove, onSuccess: publish });
  const restart = useMutation({ mutationFn: commands.desktopRestart });
  if (!status.data) {
    return (
      <SettingsRow label="Organization Pack" error={status.error ? message(status.error) : null}>
        <span className="text-sm text-muted-foreground" role="status">
          {status.error ? "Unavailable" : "Reading the organization pack…"}
        </span>
      </SettingsRow>
    );
  }
  return (
    <OrganizationPackView
      presentation={organizationPackPresentation(status.data)}
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

export function OrganizationPackView({ presentation, choiceError, busy, choose, remove, restart }: OrganizationPackViewProps) {
  return (
    <SettingsRow label="Organization Pack" description={presentation.detail} error={choiceError ?? presentation.error}>
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

function message(error: unknown): string {
  return error instanceof Error ? error.message : typeof error === "string" ? error : "The organization pack could not be changed.";
}
