/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod aliased_result;
pub(crate) mod identities;
pub(crate) mod renamed_sdk;

/* === Re-Exports === */

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

/* === Private Functions === */

/* === Tests === */

#[test]
fn domain_contract() {
    {
        eprintln!("Scenario: aliased_result::aliased_result_template");
        aliased_result::aliased_result_template();
    }
    {
        eprintln!("Scenario: identities::item_names_identities");
        identities::item_names_identities();
    }
    {
        eprintln!("Scenario: renamed_sdk::alias_compiles_operations");
        renamed_sdk::alias_compiles_operations();
    }
    {
        eprintln!("Scenario: renamed_sdk::compiled_authoring_contracts");
        renamed_sdk::compiled_authoring_contracts();
    }
}
