/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// What `ListConnections` answers.
#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Connections {
    #[row(description = "The resource ids, one per unit the listener heard about.")]
    pub resources: Vec<String>,
}
