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

product_error!(ParameterLimitError, formatter {
 Self::Structure => write!(formatter,"parameter value/expression structural budget exceeded"),
 Self::Expansion => write!(formatter,"parameter expansion exceeds limits or cannot advance safely"),
 Self::Invalid(message) => write!(formatter,"{message}"),
});

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(Debug, PartialEq)]
pub enum ParameterLimitError {
    Structure,
    Expansion,
    Invalid(&'static str),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
