/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */
#[macro_export]
macro_rules! trace {
    ($ctx:expr, $($arg:tt)+) => {
        $ctx.trace(::std::format!($($arg)+))
    };
}

#[macro_export]
macro_rules! debug {
    ($ctx:expr, $($arg:tt)+) => {
        $ctx.debug(::std::format!($($arg)+))
    };
}

#[macro_export]
macro_rules! info {
    ($ctx:expr, $($arg:tt)+) => {
        $ctx.info(::std::format!($($arg)+))
    };
}

#[macro_export]
macro_rules! warn {
    ($ctx:expr, $($arg:tt)+) => {
        $ctx.warn(::std::format!($($arg)+))
    };
}

#[macro_export]
macro_rules! error {
    ($ctx:expr, $($arg:tt)+) => {
        $ctx.error(::std::format!($($arg)+))
    };
}

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Entry {
    #[row(description = "Numeric severity on the wire; producers use [`Level`].")]
    pub level: i32,
    pub message: String,
    #[row(description = "UTC RFC 3339 time at publication, with millisecond precision.")]
    pub timestamp: String,
}

/* === Enums === */
#[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
#[repr(i32)]
pub enum Level {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

/* === Traits === */

/* === Implementations === */
impl Entry {
    pub fn new(level: Level, message: impl Into<String>) -> Self {
        Self { level: level as i32, message: message.into(), timestamp: timestamp() }
    }
}

/* === Public Functions === */
pub fn timestamp() -> String {
    let now = crate::model::time::now_utc();
    format!("{}.{:03}Z", &now.as_str()[..19], now.unix_millis().unwrap_or_default() % 1000)
}

/* === Private Functions === */

/* === Tests === */
