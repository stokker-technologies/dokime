/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { TestCase } from "@/lib/types";

export function testCaseTitle(testCase: TestCase | undefined, fallback: string): string {
  const label = testCase?.label?.trim();
  if (!label) {
    return fallback;
  }
  const parameterStart = label.indexOf(" (");
  return parameterStart > 0 ? label.slice(0, parameterStart) : label;
}
