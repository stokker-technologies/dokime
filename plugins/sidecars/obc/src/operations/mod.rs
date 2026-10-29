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

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
pub enum Operations {
    #[nested]
    Bench(crate::domain::operations::Operations),

    #[nested]
    Analyses(crate::analyses::Analyses),
}
