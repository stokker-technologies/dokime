/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


use dokime::prelude::StatusResponse;


// Each request is its operation: the struct is what a caller sends, and the
// attribute is everything the host reads about it before the plugin runs.
#[dokime::operation(
    id = "{{crate}}.get_status",
    response = StatusResponse,
    description = "Reports whether the bench is reachable.",
    policy = super::POLICY,
    resources = [crate::Bench]
)]
#[derive(Clone, PartialEq)]
pub struct GetStatus {}
