/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The Folders the Built-In Host Lets Plugins Capture Device Files From. */

/* === External Imports === */

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useId, useState } from "react";

/* === Local Imports === */

import { ErrorCard } from "@/components/ErrorCard";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { commands } from "@/lib/desktop/commands";
import { failureOf } from "@/lib/notification-details";
import { SettingsCard } from "../SettingsCard";
import { SettingsPathField } from "../SettingsPathField";
import { SettingsRow } from "../SettingsRow";

/* === Constants === */

const QUERY_KEY = ["settings", "capture-folders"];

/* === Components === */

export function CaptureFoldersSection() {
  const queryClient = useQueryClient();
  const inputId = useId();
  const [draft, setDraft] = useState("");
  const query = useQuery({ queryKey: QUERY_KEY, queryFn: () => commands.desktopCaptureFoldersRead() });
  const mutation = useMutation({
    mutationFn: (folders: string[]) => commands.desktopCaptureFoldersWrite(folders),
    onSuccess: (folders) => queryClient.setQueryData(QUERY_KEY, folders),
  });
  const folders = query.data ?? [];
  const candidate = draft.trim();
  const save = (next: string[]) => mutation.mutate(next);

  return (
    <SettingsCard
      title="Capture Folders"
      description="Plugins on this desktop's built-in host may copy device files only from these folders. Changes apply to captures started afterwards."
    >
      {query.isPending ? (
        <p role="status" className="px-4 py-3 text-sm text-muted-foreground">
          Loading Capture Folders…
        </p>
      ) : null}
      {query.error ? <ErrorCard failure={failureOf(query.error, "Capture Folders Unavailable")} /> : null}
      {query.isSuccess && folders.length === 0 ? (
        <p className="px-4 py-3 text-sm text-muted-foreground">No Capture Folders; Every Capture Is Refused.</p>
      ) : null}
      {folders.map((folder) => (
        <SettingsRow key={folder} label="Folder">
          <SettingsPathField value={folder} openable />
          <Button
            aria-label={`Remove ${folder}`}
            disabled={mutation.isPending}
            onClick={() => save(folders.filter((item) => item !== folder))}
            size="sm"
            variant="ghost"
          >
            Remove
          </Button>
        </SettingsRow>
      ))}
      <SettingsRow
        htmlFor={inputId}
        label="Add Folder"
        description="An absolute folder path on this computer."
        error={mutation.error ? failureOf(mutation.error, "Save Capture Folders").message : undefined}
        failure={mutation.error ?? undefined}
        status={mutation.isPending ? <span role="status">Saving</span> : null}
      >
        <Input id={inputId} value={draft} onChange={(event) => setDraft(event.target.value)} placeholder="D:\Bench\Device" />
        <Button
          disabled={!candidate || folders.includes(candidate) || mutation.isPending}
          onClick={() => {
            save([...folders, candidate]);
            setDraft("");
          }}
          size="sm"
        >
          Add Folder
        </Button>
      </SettingsRow>
    </SettingsCard>
  );
}
