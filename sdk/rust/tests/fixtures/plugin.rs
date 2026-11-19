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
use dokime::Plugin;
use dokime::operations::{Context, Operation};
use dokime::{Result, Visibility};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct ExamplePlugin {
    greeting: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<()> {
    let ping = Operation::new("example.ping", Visibility::Public, async |ctx: &Context, request: String| {
        let plugin = ctx.plugin::<ExamplePlugin>()?;
        if plugin.greeting != "Ready" {
            return Err(dokime::Error::Invalid("Invoke Echo: Initialize the Plugin State".into()));
        }
        Ok(request)
    });
    let plugin = Plugin::of::<ExamplePlugin>("0.1.0").state(ExamplePlugin { greeting: "Ready".into() }).operation(ping);
    plugin.run()
}

/* === Tests === */
