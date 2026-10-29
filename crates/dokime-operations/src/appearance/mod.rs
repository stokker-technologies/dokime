/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod get_appearance;

pub use get_appearance::GetAppearance;

use super::Endpoint;
use super::ORDINARY;
use dokime_domain::model::appearance::wire::*;

#[harmos::catalog]
pub enum Operations {
    GetAppearance(GetAppearance),
}

impl Endpoint for GetAppearance {
    const PATH: &'static str = "/dokime.v1.Appearance/GetAppearance";
}
