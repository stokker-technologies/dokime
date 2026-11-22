// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ReactNode } from "react";
import type { ComponentProps } from "svelte";
import { SvelteMount, useSvelteSlots } from "../../components/SvelteMount";
import ExecutionSection from "./ExecutionSectionCard.svelte";

type Props = Omit<ComponentProps<typeof ExecutionSection>, "action" | "children" | "class"> & {
  action?: ReactNode;
  children?: ReactNode;
  className?: string;
};

export function ExecutionSectionCard({ action, children, className, ...props }: Props) {
  const { rendered, portals } = useSvelteSlots({ action, children });
  return (
    <>
      <SvelteMount component={ExecutionSection} props={{ ...props, ...rendered, class: className }} />
      {portals}
    </>
  );
}
