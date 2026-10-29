/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) mod system;

pub(crate) use system::System;

#[dokime::catalog]
pub enum Observers {
    System(System),
}
