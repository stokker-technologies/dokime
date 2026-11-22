/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { useEffect, useRef, useState } from "react";
import { type PromptValues, promptDefaults } from "@/lib/prompts";
import type { PromptRequest } from "@/lib/types";

/** Polls refresh the request object; only a different prompt replaces the draft. */
export function usePromptValues(prompt: PromptRequest | undefined) {
  const correlationId = useRef(prompt?.correlation_id);
  const [values, setValues] = useState<PromptValues>(() => promptDefaults(prompt));

  useEffect(() => {
    if (correlationId.current === prompt?.correlation_id) {
      return;
    }
    correlationId.current = prompt?.correlation_id;
    setValues(promptDefaults(prompt));
  }, [prompt]);

  return [values, setValues] as const;
}
