/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

macro_rules! product_error {
    ($error:ty, $formatter:ident { $($pattern:pat => $display:expr),+ $(,)? }) => {
        impl std::fmt::Display for $error {
            fn fmt(&self, $formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $($pattern => $display),+
                }
            }
        }

        impl std::error::Error for $error {}
    };
}

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
