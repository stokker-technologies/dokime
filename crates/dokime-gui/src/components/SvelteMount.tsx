// Dokime — SPDX-License-Identifier: Apache-2.0
import { useContext, useLayoutEffect, useRef } from "react";
import type { Component } from "svelte";
import { ErrorRecoveryContext } from "./ErrorCard";
import { recoveryContext } from "./error-recovery";
import { mountSvelte } from "./mount-svelte.svelte";

export function SvelteMount<Props extends Record<string, unknown>>({ component, props }: { component: Component<Props>; props: Props }) {
  const resolveRecovery = useContext(ErrorRecoveryContext);
  const host = useRef<HTMLDivElement>(null);
  const rendered = useRef<ReturnType<typeof mountSvelte<Props>> | null>(null);
  const current = useRef(props);

  useLayoutEffect(() => {
    current.current = props;
    rendered.current?.update(props);
  }, [props]);

  useLayoutEffect(() => {
    if (!host.current) {
      return;
    }
    const context = new Map([[recoveryContext, resolveRecovery]]);
    const instance = mountSvelte(component, host.current, current.current, context);
    rendered.current = instance;
    return () => {
      rendered.current = null;
      instance.destroy();
    };
  }, [component, resolveRecovery]);

  return <div ref={host} className="contents" />;
}
