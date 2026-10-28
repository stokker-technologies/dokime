/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

const PATH_KEY_PATTERN = /(^|_)(dir|path|directory|file|folder)(_|$)/i;
const WINDOWS_PATH_PATTERN = /^[a-z]:[\\/][^<>:"|?*\n\r]+/i;
const UNC_PATH_PATTERN = /^\\\\[^\\/\n\r]+[\\/][^\\/\n\r]+/;
const POSIX_PATH_PATTERN = /^\/(?:[^/\n\r]+\/)+[^/\n\r]*/;

export function isPathLike(value: string, key = ""): boolean {
  const trimmed = value.trim();
  if (!trimmed || trimmed === "-") {
    return false;
  }
  if (PATH_KEY_PATTERN.test(key) && hasPathSeparator(trimmed)) {
    return true;
  }
  return WINDOWS_PATH_PATTERN.test(trimmed) || UNC_PATH_PATTERN.test(trimmed) || POSIX_PATH_PATTERN.test(trimmed);
}

export function compactPath(value: string): string {
  const trimmed = value.trim();
  const separator = trimmed.includes("\\") ? "\\" : "/";
  const parts = trimmed.split(/[\\/]+/).filter(Boolean);
  if (parts.length <= 3) {
    return trimmed;
  }
  return `...${separator}${parts.slice(-2).join(separator)}`;
}

function hasPathSeparator(value: string): boolean {
  return value.includes("\\") || value.includes("/");
}
