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

pub use crate::operations::execution::transport::{Entry, Level, timestamp};

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

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
