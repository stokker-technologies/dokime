// Dokime — SPDX-License-Identifier: Apache-2.0

/* The page owns its header. Its title reaches the shell as a value, and its
 * actions render inside the page's own tree, moved into the shell's header
 * slot. The shell never evaluates page state, so a page whose props go stale
 * while it is being replaced cannot break the shell's update.
 */

let title = $state.raw<{ value: string } | null>(null);
let slot = $state.raw<HTMLElement | null>(null);

export function publishPageTitle(read: () => string) {
  $effect(() => {
    const owned = { value: read() };
    title = owned;
    return () => {
      if (title === owned) {
        title = null;
      }
    };
  });
}

export function pageTitle(): string | undefined {
  return title?.value;
}

/** Attach to the shell element that receives page actions. */
export function pageActionsSlot(node: HTMLElement) {
  slot = node;
  return () => {
    if (slot === node) {
      slot = null;
    }
  };
}

/** Attach to a page element to show it in the shell's header slot. */
export function moveToPageActions(node: HTMLElement) {
  if (!slot) {
    return;
  }
  slot.append(node);
  return () => node.remove();
}

export function pageActionsReady(): boolean {
  return slot !== null;
}
