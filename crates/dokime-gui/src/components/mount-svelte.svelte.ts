// Dokime — SPDX-License-Identifier: Apache-2.0
import { type Component, mount, unmount } from "svelte";

function reactiveProp(initial?: unknown) {
  let value = $state.raw(initial);
  return {
    get: () => value,
    set(next: unknown) {
      value = next;
    },
  };
}

export function mountSvelte<Props extends Record<string, unknown>>(
  component: Component<Props>,
  target: HTMLElement,
  initial: Props,
  context?: Map<unknown, unknown>,
) {
  // React Freezes Props In Development. Track Each Prop Independently So
  // A Callback Can Update React State Without Invalidating Its Own Inputs.
  const initialKeys = Reflect.ownKeys(initial);
  const props = new Map(initialKeys.map((key) => [key, reactiveProp(Reflect.get(initial, key))]));
  let keys = $state.raw(initialKeys);

  function prop(key: string | symbol) {
    let held = props.get(key);
    if (!held) {
      held = reactiveProp();
      props.set(key, held);
    }
    return held;
  }

  const reactiveProps = new Proxy({} as Props, {
    get(_, key) {
      return prop(key).get();
    },
    has(_, key) {
      return keys.includes(key);
    },
    ownKeys() {
      return keys;
    },
    getOwnPropertyDescriptor(_, key) {
      return keys.includes(key) ? { configurable: true, enumerable: true } : undefined;
    },
  });
  const instance = mount(component, { target, props: reactiveProps, context });

  return {
    update(next: Props) {
      const nextKeys = Reflect.ownKeys(next);
      for (const key of keys) {
        if (!Object.hasOwn(next, key)) {
          prop(key).set(undefined);
        }
      }
      for (const key of nextKeys) {
        prop(key).set(Reflect.get(next, key));
      }
      if (keys.length !== nextKeys.length || keys.some((key, index) => key !== nextKeys[index])) {
        keys = nextKeys;
      }
    },
    destroy() {
      void unmount(instance);
    },
  };
}
