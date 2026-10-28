/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

declare const __APP_VERSION__: string;
declare const __DOKIME_UI_MODE__: "live" | "report";

declare module "*.css" {}

/** Vite's build mode flag: true only while the development server serves the frontend. */
interface ImportMetaEnv {
  readonly DEV: boolean;
}
interface ImportMeta {
  readonly env: ImportMetaEnv;
}
