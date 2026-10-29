/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

export type UiSurfaceMode = "live" | "report";

export function currentUiSurfaceMode(): UiSurfaceMode {
  /* Read the define rather than the module constant: the build folds this to a
     literal, and a test can still say which surface it is rendering. */
  return __DOKIME_UI_MODE__;
}

/**
 * Whether this bundle is the frozen single-file report.
 *
 * The one source of truth for "there is no desktop here". A report is opened
 * from a `file://` URL, sometimes inside the Tauri WebView, where
 * `window.__TAURI_INTERNALS__` exists and every desktop command would be
 * answered — so presence of the internals is not a surface test. Nothing in a
 * report build may reach `invoke`, and this is what every guard asks.
 */
export function isReportBuild(): boolean {
  return currentUiSurfaceMode() === "report";
}
