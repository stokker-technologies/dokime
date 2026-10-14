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
use crate::report::input::CycleReportInput;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn document_warnings(input: &CycleReportInput) -> impl Iterator<Item = (&str, &str)> {
    input
        .documents
        .values()
        .chain(
            input
                .case_documents
                .iter()
                .flat_map(|case| case.documents.values()),
        )
        .flat_map(|document| document.warnings.iter())
        .map(|warning| (warning.code.as_str(), warning.message.as_str()))
        .chain(
            input
                .omissions
                .iter()
                .map(|omission| ("report.omitted", omission.as_str())),
        )
}

/* === Private Functions === */

/* === Tests === */
