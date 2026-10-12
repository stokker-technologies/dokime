/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Content view models shared by native adapters and the original UI. */

export type ExecutionFileTextRead = {
  text: string;
  offset: number;
  sizeBytes: number;
  nextOffset: number;
  truncated: boolean;
  rotated: boolean;
};
