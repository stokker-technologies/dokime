/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#[cfg(windows)]
pub mod windows;

#[cfg(not(windows))]
pub mod linux;
