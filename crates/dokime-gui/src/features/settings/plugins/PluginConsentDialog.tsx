/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { ShieldAlert } from "lucide-react";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { formatArtifactSize } from "@/lib/artifact-tree";
import { useBrand } from "@/lib/brand";
import type { PluginEntry } from "@/lib/types/view-contracts";

/* === Types === */

type PluginConsentDialogProps = {
  busy: boolean;
  entry: PluginEntry | null;
  error: string | null;
  onCancel: () => void;
  onInstall: () => void;
};

/* === Components === */

export function PluginConsentDialog({ busy, entry, error, onCancel, onInstall }: PluginConsentDialogProps) {
  const brand = useBrand();
  return (
    <Dialog
      onOpenChange={(open) => {
        if (!open && entry && !busy) {
          onCancel();
        }
      }}
      open={Boolean(entry)}
    >
      <DialogContent className="sm:max-w-xl" showCloseButton={!busy}>
        <DialogHeader>
          <DialogTitle>Review Plugin</DialogTitle>
          <DialogDescription>
            {brand.label} inspected this file without running it. Confirm the declared identity before installing it.
          </DialogDescription>
        </DialogHeader>

        {entry ? (
          <div className="grid gap-4">
            <dl className="grid grid-cols-[minmax(7rem,auto)_minmax(0,1fr)] gap-x-4 gap-y-2 rounded-lg border p-3 text-sm">
              <Fact label="Plugin" value={entry.metadata?.identity.name ?? entry.install_id} mono />
              <Fact label="Version" value={entry.metadata?.identity.version ?? "Unavailable"} />
              <Fact label="Description" value={entry.metadata?.identity.description || "No description declared."} />
              <Fact label="Executables" value={entry.metadata?.owners.join(", ") ?? "Unavailable"} mono wrap />
              <Fact label="Installation" value={entry.install_id} />
              <Fact label="Size" value={entry.metadata ? formatArtifactSize(entry.metadata.size_bytes) : "Unavailable"} />
              <Fact label="SHA-256" value={entry.sha256} mono wrap />
            </dl>

            {entry.metadata && entry.metadata.environment_references.length > 0 ? (
              <section className="grid gap-2 rounded-lg border p-3 text-sm" aria-label="Environment references">
                <h3 className="font-medium">Environment References</h3>
                <p className="text-xs text-muted-foreground">
                  These declared settings can read the named host environment variables after installation. Values are never shown here.
                </p>
                <ul className="grid gap-2">
                  {entry.metadata.environment_references.map((reference) => (
                    <li key={`${reference.owner}:${reference.setting}:${reference.variable}`} className="break-all">
                      <span className="font-mono">{reference.owner}</span>
                      {" · "}
                      <span className="font-mono">{reference.setting}</span>
                      {" → "}
                      <span className="font-mono">{reference.variable}</span>
                    </li>
                  ))}
                </ul>
              </section>
            ) : null}

            <div className="flex gap-3 rounded-lg border border-warning/30 bg-warning/10 p-3 text-sm" role="alert">
              <ShieldAlert className="mt-0.5 size-4 shrink-0 text-warning" />
              <div className="grid gap-1">
                <p className="font-medium text-foreground">
                  Installing runs these native executables with {brand.label}&apos;s privileges at every start — install only plugins you
                  trust.
                </p>
                <p className="text-xs text-muted-foreground">
                  It will not run until the next restart. If this plugin is already installed, this bundle replaces it at that restart.
                </p>
              </div>
            </div>

            {error ? (
              <p className="text-sm text-destructive" role="alert">
                {error}
              </p>
            ) : null}
          </div>
        ) : null}

        <DialogFooter>
          <Button disabled={busy} onClick={onCancel} type="button" variant="outline">
            {busy ? "Working…" : "Cancel"}
          </Button>
          <Button disabled={busy || !entry?.metadata || Boolean(entry.error)} onClick={onInstall} type="button">
            {busy ? "Installing…" : "Install plugin"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function Fact({ label, mono = false, value, wrap = false }: { label: string; mono?: boolean; value: string; wrap?: boolean }) {
  return (
    <>
      <dt className="text-muted-foreground">{label}</dt>
      <dd className={`${mono ? "font-mono text-xs" : ""} ${wrap ? "break-all" : "min-w-0 truncate"}`} title={value}>
        {value}
      </dd>
    </>
  );
}
