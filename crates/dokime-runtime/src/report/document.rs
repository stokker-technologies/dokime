/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::report::input::CycleReportInput;

pub(crate) fn document_warnings(input: &CycleReportInput) -> impl Iterator<Item = (&str, &str)> {
    input
        .documents
        .values()
        .chain(input.case_documents.iter().flat_map(|case| case.documents.values()))
        .flat_map(|document| document.warnings.iter())
        .map(|warning| (warning.code.as_str(), warning.message.as_str()))
}
