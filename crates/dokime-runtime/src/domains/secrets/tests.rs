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
use super::{Backend, CredentialStore};

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
fn stores_isolate_values() {
    let first = CredentialStore(keyring_core::mock::Store::new().unwrap());
    let second = CredentialStore(keyring_core::mock::Store::new().unwrap());
    first.set("same-slot", "first-value").unwrap();
    assert!(second.get("same-slot").unwrap().is_none());
    second.set("same-slot", "second-value").unwrap();
    assert_eq!(first.get("same-slot").unwrap().unwrap().expose(), "first-value");
    assert_eq!(second.get("same-slot").unwrap().unwrap().expose(), "second-value");

    first.remove("same-slot").unwrap();
    assert!(first.get("same-slot").unwrap().is_none());
    assert_eq!(second.get("same-slot").unwrap().unwrap().expose(), "second-value");
    assert_ne!(super::SHARED_SERVICE, super::LEGACY_SERVICE);
}

#[test]
fn request_redacts_password() {
    let password = "correct horse battery staple";
    let names = vec!["alpha".into(), "beta".into()];
    let request = dokime_domain::operations::secrets::Export { names, password: password.into(), generation: 2 };
    assert!(!format!("{request:?}").contains(password));
}
