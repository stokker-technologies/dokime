// Dokime — SPDX-License-Identifier: Apache-2.0
import { type Component, mount, unmount } from "svelte";

export function mountSvelte<Props extends Record<string, unknown>>(
  component: Component<Props>,
  target: HTMLElement,
  initial: Props,
  context?: Map<unknown, unknown>,
) {
  let props = $state.raw(initial);
  const reactiveProps = new Proxy(initial, {
    get(_, key) {
      return Reflect.get(props, key);
    },
  });
  const instance = mount(component, { target, props: reactiveProps, context });

  return {
    update(next: Props) {
      props = next;
    },
    destroy() {
      void unmount(instance);
    },
  };
}
