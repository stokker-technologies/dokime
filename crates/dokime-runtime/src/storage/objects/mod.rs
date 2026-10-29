/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod annotations;
mod artifacts;
pub mod case;
pub mod cycle;
pub mod execution;
mod metadata;
pub mod presentation;
pub mod suite;
mod templates;

#[crate::catalog]
pub enum Objects {
    Case(case::Case),
    Command(super::requests::Command),
    Cycle(cycle::Cycle),
    Execution(execution::Execution),
    Request(super::requests::Request),
    Suite(suite::Suite),
}
