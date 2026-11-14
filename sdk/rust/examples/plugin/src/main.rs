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
use dokime::plugin::{Context, Operation, Plugin};
use dokime::{Message, Result, Visibility};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Message)]
struct Ping {
    value: String,
}

#[derive(Message)]
struct Pong {
    value: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<()> {
    let ping = Operation::new("example.ping", Visibility::Public, async |_: &Context, request: Ping| Ok(Pong { value: request.value }));
    let plugin = Plugin::new("example", "0.1.0").operation(ping);
    let plugin = if std::env::args().any(|argument| argument == "--remote") { plugin.remote() } else { plugin };
    plugin.serve()
}

/* === Tests === */
