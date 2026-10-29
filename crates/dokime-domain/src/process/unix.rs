/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::process::Command;

// Only Windows Opens A Console Window Per Child.
pub(super) fn without_console(_command: &mut Command) {}
