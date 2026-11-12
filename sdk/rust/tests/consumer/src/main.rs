/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Public SDK Consumer

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Message)]
struct Reading {
    celsius: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> dokime::Result<()> {
    use dokime::prelude::*;
    let stream = Stream::<Reading>::new("consumer.temperature", "Temperature [°C]");
    Plugin::new("consumer", "1.0.0").stream(stream).catalog()?;
    Ok(())
}

/* === Tests === */
