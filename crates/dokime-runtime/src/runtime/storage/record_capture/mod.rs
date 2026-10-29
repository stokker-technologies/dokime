/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod source;

pub use source::CapturedRecord;
pub(crate) use source::{Process, Projection, Source};

use crate::runtime::Error;

fn error(value: impl std::fmt::Display) -> Error {
    Error::Operation(format!("record capture: {value}"))
}
