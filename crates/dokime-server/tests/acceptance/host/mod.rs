/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod hub_log;
mod terminal;

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

/* === Private Functions === */
fn executable() -> std::path::PathBuf {
    let path = std::path::PathBuf::from(
        std::env::var_os("DOKIME_TEST_HOST").expect("mise selects the exact release host"),
    );
    assert!(
        path.is_file(),
        "run mise run build:release first: {}",
        path.display()
    );
    path
}

/* === Tests === */
