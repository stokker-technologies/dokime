/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub mod connections;
pub mod device;
pub mod readiness;
pub mod status;

#[dokime::catalog]
pub enum Operations {
    #[nested]
    Bench(crate::domain::operations::Operations),

    #[nested]
    Analyses(crate::analyses::Analyses),
}
