/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Describe the Plugin Without a Host

/* === Modules === */



/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::{Result, Suites};
use dokime::encoding::Message;
use starter_project::plugin::plugin;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */



/* === Enums === */

/* === Traits === */

/* === Implementations === */



/* === Public Functions === */



/* === Private Functions === */



/* === Tests === */

#[test]
fn plugin_describes_its_confirmation_suite() -> Result<()> {
    let declarations = plugin()?.declarations()?;
    let section = declarations.sections.iter().find(|section| section.r#type.ends_with(".Suites")).expect("Declared Suites");
    let suites = Suites::decode(section.payload.as_slice()).map_err(|error| dokime::Error::Invalid(error.to_string()))?;
    assert_eq!(suites.suites.len(), 1);
    assert_eq!(suites.suites[0].templates[0].metadata.as_ref().unwrap().identifier, "capture.confirmation");
    Ok(())
}
