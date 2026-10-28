/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::device::Device;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Testbench {
    pub stopped: bool,
    pub device: Device,
}
