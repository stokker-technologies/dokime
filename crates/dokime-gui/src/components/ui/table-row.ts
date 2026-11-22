// Dokime — SPDX-License-Identifier: Apache-2.0

/** Pointer Activation On The Row Complements Its Keyboard-Accessible Primary Button. */
export function activateTableRow(row: HTMLTableRowElement, onActivate: () => void) {
  function activate(event: MouseEvent) {
    const target = event.target;
    if (target instanceof Element && !target.closest("button, a, input, [role=menuitem]")) {
      onActivate();
    }
  }

  row.addEventListener("click", activate);
  return () => row.removeEventListener("click", activate);
}
