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

#[dokime::catalog]
pub enum Listeners {
    Observed(Observed),
}
