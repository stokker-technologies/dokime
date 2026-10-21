/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * One-Time Access Key Dialog
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { useState } from "react";

// Local Imports
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Textarea } from "@/components/ui/textarea";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type AccessKeyDialogProps = {
  value: string;
  busy: boolean;
  saved: boolean;
  saveError: string | null;
  close: () => void;
  save: () => void;
  /** Absent when the host has no public certificate; the action is then hidden, not disabled. */
  saveCertificate: (() => void) | undefined;
};

/* === Components === */

export function AccessKeyDialog(props: AccessKeyDialogProps) {
  const { busy, close } = props;
  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open && !busy) close();
      }}
    >
      <DialogContent className="sm:max-w-lg" showCloseButton={!busy}>
        <AccessKeyPanel {...props} />
      </DialogContent>
    </Dialog>
  );
}

/** The dialog body, separate from the portal so it renders without a document. */
export function AccessKeyPanel({ value, busy, saved, saveError, close, save, saveCertificate }: AccessKeyDialogProps) {
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState<string | null>(null);
  return (
    <>
      <DialogHeader>
        <DialogTitle>Save Your Access Key</DialogTitle>
        <DialogDescription>
          This is the only time the key is shown. It gives full control of this host. Copy it or save it as a key file before closing.
        </DialogDescription>
      </DialogHeader>
      <Textarea
        aria-label="Generated Access Key"
        autoComplete="off"
        readOnly
        rows={2}
        value={value}
        className="min-w-0 resize-none break-all font-mono text-xs md:text-xs"
      />
      {error || saveError ? <p role="alert">{error ?? saveError}</p> : null}
      {saved || copied ? <p role="status">{saved ? "Key File Saved" : "Key Copied"}</p> : null}
      <DialogFooter>
        <Button
          variant="outline"
          onClick={async () => {
            try {
              await navigator.clipboard.writeText(value);
              setCopied(true);
            } catch {
              setError("Copy failed; save the key file or select and copy the key.");
            }
          }}
        >
          {copied ? "Copied" : "Copy Key"}
        </Button>
        <Button variant="outline" disabled={busy} onClick={save}>
          {busy ? "Saving…" : "Save Key File…"}
        </Button>
        {saveCertificate ? (
          <Button variant="outline" disabled={busy} onClick={saveCertificate}>
            Save Public Certificate…
          </Button>
        ) : null}
        <Button disabled={busy} onClick={close}>
          I Have Saved The Key
        </Button>
      </DialogFooter>
    </>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
