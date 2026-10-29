/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// In-process packaging stories, one module per packaging domain.

mod bundle;
mod installer;
#[cfg(windows)]
mod resources;
