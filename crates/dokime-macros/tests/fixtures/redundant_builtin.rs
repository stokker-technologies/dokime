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

/* === Type Aliases === */

/* === Structs === */

struct Probe;

/* === Enums === */

#[sdk::catalog]
enum Builtins {
    Watch(sdk::operations::files::WatchFile),
}

#[sdk::catalog]
enum Required {
    #[nested]
    Builtins(Builtins),
}

/* === Traits === */

/* === Implementations === */

#[sdk::template(id = "redundant.checks.probe", label = "Probe", requires = Required)]
impl Probe {
    async fn initialize(_: &mut sdk::execution::InitializationContext<'_>) -> Result<Self, sdk::Error> {
        Ok(Self)
    }

    #[sdk::step(label = "Run")]
    async fn run(&mut self, _: &mut sdk::execution::StepContext<'_>) -> Result<(), sdk::Error> {
        Ok(())
    }
}

/* === Public Functions === */

fn main() {}

/* === Private Functions === */

/* === Tests === */
