/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native settings and plugin API payloads.
export type StorageSource = {
  identifier: string;
  adapter: string;
  priority: number;
  enabled: boolean;
};
