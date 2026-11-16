/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Use the Native Credential Service's Own Concurrency Control

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn call<T>(entry: &keyring::Entry, operation: impl FnOnce(&keyring::Entry) -> keyring::Result<T>) -> keyring::Result<T> {
    operation(entry)
}

/* === Private Functions === */

/* === Tests === */
