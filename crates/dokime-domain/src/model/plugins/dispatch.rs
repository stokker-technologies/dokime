/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

product_error!(ActionDispatcherError, formatter {
    Self::Refused { code, message } => write!(formatter, "action was refused: {message} ({code})"),
    Self::Unavailable => formatter.write_str("action dispatcher is unavailable"),
    Self::Deadline => formatter.write_str("action dispatcher exceeded its deadline"),
    Self::Malformed(message) => write!(formatter, "action reply was malformed: {message}"),
    Self::InvalidInput(message) => write!(formatter, "action input is invalid: {message}"),
    Self::UnknownEntrypoint(entrypoint) => write!(formatter, "action entrypoint `{entrypoint}` is unknown"),
});

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionDispatcherError {
    Refused { code: i64, message: String },
    Unavailable,
    Deadline,
    Malformed(String),
    InvalidInput(String),
    UnknownEntrypoint(String),
}
