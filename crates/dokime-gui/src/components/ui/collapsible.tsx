/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import * as radix from "radix-ui";
import * as React from "react";

function Collapsible({ ...props }: React.ComponentProps<typeof radix.Collapsible.Root>) {
  return <radix.Collapsible.Root data-slot="collapsible" {...props} />;
}

const CollapsibleTrigger = React.forwardRef<
  React.ElementRef<typeof radix.Collapsible.CollapsibleTrigger>,
  React.ComponentPropsWithoutRef<typeof radix.Collapsible.CollapsibleTrigger>
>(({ ...props }, ref) => <radix.Collapsible.CollapsibleTrigger data-slot="collapsible-trigger" ref={ref} {...props} />);

CollapsibleTrigger.displayName = radix.Collapsible.CollapsibleTrigger.displayName;

function CollapsibleContent({ ...props }: React.ComponentProps<typeof radix.Collapsible.CollapsibleContent>) {
  return <radix.Collapsible.CollapsibleContent data-slot="collapsible-content" {...props} />;
}

export { Collapsible, CollapsibleContent, CollapsibleTrigger };
