/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { createFileRoute, useRouter } from "@tanstack/react-router";
import { useCallback } from "react";

/* === Local Imports === */

import { SettingsSurface } from "@/features/settings/SettingsSurface";

/* === Route === */

export const Route = createFileRoute("/settings_/$section")({
  component: SettingsSectionRoute,
});

/* === Components === */

function SettingsSectionRoute() {
  const router = useRouter();
  const { section } = Route.useParams();
  const close = useCallback(() => {
    void router.navigate({ to: "/cycles" });
  }, [router]);
  return <SettingsSurface close={close} section={section} />;
}
