/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { commands } from "@/lib/desktop/commands";

export type {
  GitlabAuthStatus,
  GitlabInstallResult,
  GitlabUpdateCheck,
  GitlabUpdateStatus,
  GitlabUser as GitlabUserSummary,
} from "@/lib/desktop/gitlab";

export const gitlabAuthStatus = commands.gitlabAuthStatus;
export const gitlabSignIn = commands.gitlabSignIn;
export const gitlabSignOut = commands.gitlabSignOut;
export const gitlabCheckForUpdates = commands.gitlabCheckForUpdates;
export const gitlabInstallUpdate = commands.gitlabInstallUpdate;
