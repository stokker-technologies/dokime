/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// What `AskReadiness` answers.
#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Readiness {
    #[row(description = "Whether the embedded runtime has stopped taking new work.")]
    pub stopped: bool,
}
