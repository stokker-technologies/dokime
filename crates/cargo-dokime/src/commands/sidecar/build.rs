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

pub use dokime_packaging::build::Options as Args;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn run(args: Args) -> Result<(), crate::error::Error> {
    if args.from_cargo_json.is_some() {
        let collected = dokime_packaging::build::collect(&args).await?;
        if args.json {
            println!("{}", serde_json::to_string(&collected)?);
        } else {
            for (name, (path, _)) in &collected.executables {
                println!("{name}: {}", path.display());
            }
        }
        return Ok(());
    }
    println!("{}", dokime_packaging::build::run(args).await?.display());
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
