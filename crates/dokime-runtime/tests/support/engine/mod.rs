/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Shared story support: environment owners every layer of this crate reuses.

#[cfg(feature = "storage")]
pub mod local_storage;
