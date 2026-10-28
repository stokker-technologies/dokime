/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// TODO: Describe this module's responsibility.

mod authoring;

#[cfg(feature = "host")]
pub mod captured;

pub use authoring::*;

#[cfg(feature = "host")]
pub use captured::{ExecutionContext, ExecutionCycleContext};
