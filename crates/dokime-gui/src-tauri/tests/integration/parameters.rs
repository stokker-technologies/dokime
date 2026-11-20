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
use std::fs;

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
fn replacement_keeps_parameters() {
    use dokime_gui::settings::{self, Store, Value, Values};
    let root = tempfile::tempdir().unwrap();
    let store = Store::new(settings::path(root.path()));
    let save = |values: Values| {
        store.update(|saved| {
            saved.parameters = Some(values);
            Ok(())
        })
    };
    assert_eq!(settings::load(store.path()).unwrap().parameters, None);
    let values: Values = [(
        "template".into(),
        [
            (
                "parameter".into(),
                Value {
                    raw: "12".into(),
                    value_type: "int".into(),
                    unit: None,
                },
            ),
            (
                "defined".into(),
                Value {
                    raw: "115".into(),
                    value_type: "int".into(),
                    unit: Some("V".into()),
                },
            ),
        ]
        .into(),
    )]
    .into();
    save(values.clone()).unwrap();
    assert_eq!(settings::load(store.path()).unwrap().parameters, Some(values));
    let saved = fs::read(store.path()).unwrap();
    let invalid = [(
        "template".into(),
        [(
            "parameter".into(),
            Value {
                raw: "x".repeat(8193),
                value_type: "str".into(),
                unit: None,
            },
        )]
        .into(),
    )]
    .into();
    assert!(save(invalid).is_err());
    assert_eq!(fs::read(store.path()).unwrap(), saved);
    save(Values::new()).unwrap();
    assert_eq!(settings::load(store.path()).unwrap().parameters, Some(Values::new()));
    let unsupported = br#"{"parameters":{"template":{"parameter":"12"}}}"#;
    fs::write(store.path(), unsupported).unwrap();
    assert!(settings::load(store.path()).is_err());
    assert!(save(Values::new()).is_err());
    assert_eq!(fs::read(store.path()).unwrap(), unsupported);
    fs::write(store.path(), b"broken").unwrap();
    assert!(settings::load(store.path()).is_err());
    assert_eq!(fs::read(store.path()).unwrap(), b"broken");
    fs::write(store.path(), vec![b' '; 2 * 1024 * 1024]).unwrap();
    assert!(settings::load(store.path()).is_err());
}
