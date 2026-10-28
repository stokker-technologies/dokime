/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub use crate::operations::execution::transport::{Entry, Level, timestamp};

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
