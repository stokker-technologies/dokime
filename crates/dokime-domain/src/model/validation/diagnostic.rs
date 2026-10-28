/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::fmt::{Arguments, Write};

struct Count(usize);

impl Write for Count {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        self.0 = self.0.saturating_add(value.len());
        Ok(())
    }
}

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
