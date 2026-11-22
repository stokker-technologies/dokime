/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Inline Failure Presentation
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { AlertCircle } from "lucide-react";
import { createContext, useContext, useEffect, useState } from "react";

// Local Imports
import { FailureDetails } from "./FailureDetails";
import { Button } from "./ui/button";
import { routeFailure } from "../lib/failure-routing";
import { failureOf, failureText, isWaitingFailure, type NotificationFailure } from "../lib/notification-details";
import { toast } from "../lib/notifications";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export const ErrorRecoveryContext = createContext<((failure: NotificationFailure) => (() => Promise<void>) | undefined) | null>(null);

type ErrorCardProps = {
  retryFirst?: boolean;
  notificationContext?: "action" | "background";
  failure: NotificationFailure;
  onRetry?: () => void;
  retryLabel?: string;
  retryDisabled?: boolean;
};

/* === Components === */

export function ErrorCard({
  failure,
  onRetry,
  retryLabel = "Reload",
  retryDisabled = false,
  notificationContext,
  retryFirst = false,
}: ErrorCardProps) {
  const recovery = useContext(ErrorRecoveryContext)?.(failure);
  const [recovering, setRecovering] = useState(false);
  const recoveryLabel = retryFirst && onRetry ? retryLabel : recovery ? (failure.action ?? retryLabel) : onRetry ? retryLabel : "Reload";
  const recoverable = Boolean(onRetry || recovery || !failure.action || failure.action === "Reload");
  async function recover() {
    if (onRetry && retryLabel === recoveryLabel) return onRetry();
    if (recovery) {
      setRecovering(true);
      try {
        await recovery();
      } catch (error: unknown) {
        toast.error(failureOf(error, "Plugin Restart Failed"));
      } finally {
        setRecovering(false);
      }
      return;
    }
    window.location.reload();
  }
  useEffect(() => {
    if (notificationContext) toast.error(failure, { context: notificationContext });
    else routeFailure(failure, "view");
  }, [failure, notificationContext]);
  if (isWaitingFailure(failure))
    return (
      <p role="status" className="col-span-full w-full text-sm text-muted-foreground">
        Waiting for Plugin{failure.plugin ? ` ${failure.plugin}` : ""}
      </p>
    );
  return (
    <section
      role="alert"
      aria-label={failure.title}
      className="col-span-full grid w-full min-w-0 gap-3 rounded-md border border-destructive bg-card p-4 text-left text-card-foreground"
    >
      <div className="flex items-center gap-2">
        <AlertCircle aria-hidden="true" className="size-5 shrink-0 text-destructive" />
        <h3 className="font-sans text-sm font-semibold">{failure.title}</h3>
      </div>
      <p className="select-text whitespace-pre-wrap break-words text-sm">
        {[...new Set([failure.message, ...(failure.causes ?? [])])]
          .filter((message, index, messages) => !messages.slice(0, index).some((parent) => parent.includes(message)))
          .join("\n")}
      </p>
      {failure.action && (!recoverable || failure.action !== recoveryLabel) ? (
        <p className="whitespace-pre-wrap text-sm">{failure.action}</p>
      ) : null}
      <div className="flex gap-2">
        {recoverable ? (
          <Button type="button" variant="outline" size="sm" disabled={retryDisabled || recovering} onClick={() => void recover()}>
            {recoveryLabel}
          </Button>
        ) : null}
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={async () => {
            try {
              await navigator.clipboard.writeText(failureText(failure));
            } catch (error: unknown) {
              toast.error(failureOf(error, "Could Not Copy Failure"));
            }
          }}
        >
          Copy Details
        </Button>
      </div>
      <FailureDetails failure={failure} />
    </section>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
