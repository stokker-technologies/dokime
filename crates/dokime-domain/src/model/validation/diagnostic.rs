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
use std::fmt::{Arguments, Write};

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Count(usize);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Write for Count {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        self.0 = self.0.saturating_add(value.len());
        Ok(())
    }
}

/* === Public Functions === */

pub(crate) fn size(arguments: Arguments<'_>) -> usize {
    let mut count = Count(0);
    count.write_fmt(arguments).expect("diagnostic length formatting succeeds");
    count.0
}

pub(crate) fn text(arguments: Arguments<'_>) -> String {
    let mut text = String::with_capacity(size(arguments));
    text.write_fmt(arguments).expect("diagnostic writes into a reserved String");
    text
}

/* === Private Functions === */

/* === Tests === */
