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
/** The desktop binds `ipc`; only a browser build selects `grpc-web`. */
declare const __DOKIME_TRANSPORT__: "ipc" | "grpc-web";

declare module "*.css" {}

/** Vite's build mode flag: true only while the development server serves the frontend. */
interface ImportMetaEnv {
  readonly DEV: boolean;
}
interface ImportMeta {
  readonly env: ImportMetaEnv;
}
