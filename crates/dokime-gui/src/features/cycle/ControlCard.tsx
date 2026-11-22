// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ReactNode } from "react";
import type { ComponentProps } from "svelte";
import { SvelteMount, useSvelteSlots } from "../../components/SvelteMount";
import ControlView from "./ControlCard.svelte";

type Props = Omit<ComponentProps<typeof ControlView>, "actions" | "children"> & {
  actions?: ReactNode;
  children?: ReactNode;
};

export function ControlCard({ actions, children, ...props }: Props) {
  const { rendered, portals } = useSvelteSlots({ actions, children });
  return (
    <>
      <SvelteMount component={ControlView} props={{ ...props, ...rendered }} />
      {portals}
    </>
  );
}
