/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import type { ComponentProps } from "react";

/* === Local Imports === */

import { cn } from "../../lib/utils";
import { inputClass } from "./input-style";

/* === Components === */

function Input({ className, type, ...props }: ComponentProps<"input">) {
  return <input type={type} data-slot="input" className={cn(inputClass, className)} {...props} />;
}

export { Input };
