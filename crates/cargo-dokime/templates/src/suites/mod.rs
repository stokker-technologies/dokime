/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


pub mod soak;


#[dokime::catalog]
pub enum Suites {
    SoakSuite(soak::SoakSuite),
}
