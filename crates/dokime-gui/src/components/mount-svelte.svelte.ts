// Dokime — SPDX-License-Identifier: Apache-2.0
import { type Component, mount, unmount } from "svelte";
import { SvelteMap } from "svelte/reactivity";

export function mountSvelte<Props extends Record<string, unknown>>(
  component: Component<Props>,
  target: HTMLElement,
  initial: Props,
  context?: Map<unknown, unknown>,
) {
  // React Freezes Props In Development. Track Each Prop Independently So
  // A Callback Can Update React State Without Invalidating Its Own Inputs.
  const props = new SvelteMap<string | symbol, unknown>(Object.entries(initial));
  const reactiveProps = new Proxy({} as Props, {
    get(_, key) {
      return props.get(key);
    },
    has(_, key) {
      return props.has(key);
    },
    ownKeys() {
      return [...props.keys()];
    },
    getOwnPropertyDescriptor(_, key) {
      return props.has(key) ? { configurable: true, enumerable: true } : undefined;
    },
  });
  const instance = mount(component, { target, props: reactiveProps, context });

  return {
    update(next: Props) {
      for (const key of props.keys()) {
        if (!Object.hasOwn(next, key)) {
          props.delete(key);
        }
      }
      for (const [key, value] of Object.entries(next)) {
        props.set(key, value);
      }
    },
    destroy() {
      void unmount(instance);
    },
  };
}
