/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

export type UpdateMode = "disabled" | "manual" | "automatic_check";

export type UpdateSelection = { mode: UpdateMode; source: string | null };

export type PluginRelease = { candidate: string; package: string; version: string; target: string; requiresDokime: string };

export type UpdateCheck = {
  status: "disabled" | "available" | "up_to_date";
  currentVersion: string;
  version: string | null;
  body: string | null;
  candidate: string | null;
};

export type UpdateStatus = UpdateSelection & {
  sources: string[];
  lastCheck: UpdateCheck | null;
  error: string | null;
};
