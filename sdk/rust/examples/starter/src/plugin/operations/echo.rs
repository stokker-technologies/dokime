/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Echo Operation

/* === Modules === */



/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::operations::{Context, Operation};
use dokime::{Result, Visibility};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */



/* === Enums === */

/* === Traits === */

/* === Implementations === */



/* === Public Functions === */

pub fn operation() -> Operation {
    Operation::new("example.echo", Visibility::Public, echo)
}

/* === Private Functions === */

async fn echo(_ctx: &Context, request: String) -> Result<String> {
    Ok(request)
}

/* === Tests === */


