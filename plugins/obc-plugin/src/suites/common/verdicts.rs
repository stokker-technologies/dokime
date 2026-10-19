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

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn decided<R: dokime_runtime::records::DeclaredRecord>(
    records: dokime_runtime::Result<Vec<R>>, accepted: impl Fn(&R) -> bool,
) -> Result<bool, crate::error::Error> {
    records?.last().map(accepted).ok_or_else(|| crate::error::Error::MissingEvidence(R::RECORD_TYPE.to_owned()))
}

/* === Private Functions === */

/* === Tests === */
