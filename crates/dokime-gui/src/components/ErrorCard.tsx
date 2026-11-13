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
import { useEffect } from "react";

// Local Imports
import { FailureDetails } from "@/components/FailureDetails";
import { Button } from "@/components/ui/button";
import { routeFailure } from "@/lib/failure-routing";
import { failureOf, failureText, type NotificationFailure } from "@/lib/notification-details";
import { toast } from "@/lib/notifications";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type ErrorCardProps = { failure: NotificationFailure; onRetry?: () => void };

/* === Components === */

export function ErrorCard({ failure, onRetry }: ErrorCardProps) {
  useEffect(() => {
    routeFailure(failure, "view");
  }, [failure]);
  return (
    <section role="alert" className="grid gap-3 rounded-md border border-destructive bg-card p-4 text-card-foreground">
      <div className="flex items-center gap-2">
        <AlertCircle aria-hidden="true" className="size-5 shrink-0 text-destructive" />
        <h3 className="font-sans text-sm font-semibold">{failure.title}</h3>
      </div>
      <p className="whitespace-pre-wrap text-sm">{failure.message}</p>
      <FailureDetails failure={failure} />
      <div className="flex gap-2">
        {onRetry && (
          <Button type="button" variant="outline" size="sm" onClick={onRetry}>
            Try Again
          </Button>
        )}
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => {
            void navigator.clipboard.writeText(failureText(failure)).catch((error: unknown) => {
              toast.error(failureOf(error, "Could Not Copy Failure"));
            });
          }}
        >
          Copy
        </Button>
      </div>
    </section>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
