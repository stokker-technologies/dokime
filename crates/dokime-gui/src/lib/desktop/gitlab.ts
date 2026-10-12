/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Account login and update checks, mirroring the types beside the GitLab
   commands in `src-tauri/src/app/gitlab.rs`. */

/* === Types === */

export type GitlabUser = {
  /** GitLab sends this identifier as a JSON number. */
  id: number;
  username: string;
  name: string;
  webUrl: string | null;
};

export type GitlabAuthStatus = {
  configured: boolean;
  signedIn: boolean;
  user: GitlabUser | null;
  message: string | null;
};

export type GitlabUpdateStatus = "available" | "not_configured" | "signed_out" | "up_to_date";

export type GitlabUpdateCheck = {
  status: GitlabUpdateStatus;
  currentVersion: string;
  version: string | null;
  body: string | null;
  date: string | null;
  message: string | null;
};

export type GitlabInstallResult = {
  version: string;
};
