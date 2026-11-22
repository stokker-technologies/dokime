// Dokime — SPDX-License-Identifier: Apache-2.0
import { type ReactNode, useContext, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { type Component, createRawSnippet, type Snippet } from "svelte";
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

/** Keep remaining React children in their desktop providers while their owning view is ported. */
export function useSvelteSlots<Name extends string>(slots: Partial<Record<Name, ReactNode>>) {
  const snippets = useRef(new Map<Name, Snippet>());
  const [targets, setTargets] = useState(new Map<Name, Element>());
  const rendered: Partial<Record<Name, Snippet>> = {};
  for (const name in slots) {
    if (!slots[name]) {
      continue;
    }
    let snippet = snippets.current.get(name);
    if (!snippet) {
      snippet = createRawSnippet(() => ({
        render: () => '<div class="contents"></div>',
        setup(element) {
          setTargets((current) => new Map(current).set(name, element));
          return () => {
            setTargets((current) => {
              if (current.get(name) !== element) {
                return current;
              }
              const next = new Map(current);
              next.delete(name);
              return next;
            });
          };
        },
      }));
      snippets.current.set(name, snippet);
    }
    rendered[name] = snippet;
  }
  return { rendered, portals: [...targets].map(([name, target]) => createPortal(slots[name], target, name)) };
}
