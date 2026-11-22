// Dokime — SPDX-License-Identifier: Apache-2.0

/* === Imports === */

import { type ReactNode, useCallback } from "react";
import { ErrorRecoveryContext } from "@/components/ErrorCard";
import type { NotificationFailure } from "@/lib/notification-details";
import { PlotPaletteContext } from "@/plots/use-plot-palette";
import { TraceabilitySaveContext } from "@/components/traceability/TraceabilityJsonDialog";
import { failureRecovery } from "@/lib/failure-recovery";
import { saveNativeDocument } from "@/lib/api/native-document-delivery";
import { usePlotPalette } from "@/plots/desktop-plot-palette";
import { useNativeSessionOptional } from "./native-session/context";

/* === Types === */

type DesktopPresentationProps = { children: ReactNode };

/* === Components === */

export function DesktopPresentation({ children }: DesktopPresentationProps) {
  const session = useNativeSessionOptional();
  const palette = usePlotPalette();
  const recovery = useCallback((failure: NotificationFailure) => failureRecovery(session, failure), [session]);
  const save = useCallback(
    async (json: string, filename: string) => {
      if (!session) throw new Error("Connect to a Backend Before Saving Traceability");
      await saveNativeDocument(session.identity, { kind: "json_document", filename, bytes: new TextEncoder().encode(json) });
    },
    [session],
  );
  return (
    <ErrorRecoveryContext.Provider value={recovery}>
      <TraceabilitySaveContext.Provider value={save}>
        <PlotPaletteContext.Provider value={palette}>{children}</PlotPaletteContext.Provider>
      </TraceabilitySaveContext.Provider>
    </ErrorRecoveryContext.Provider>
  );
}
