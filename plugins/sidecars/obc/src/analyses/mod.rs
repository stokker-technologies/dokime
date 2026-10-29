/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub mod ripple;

use dokime::analysis::Operation;

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
pub enum Analyses {
    Ripple(Operation<ripple::Ripple>),
}
