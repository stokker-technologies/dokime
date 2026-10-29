/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) mod connection;

pub(crate) use connection::Observed;

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
pub enum Listeners {
    Observed(Observed),
}
