/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

//! Parameter Persistence And Atomic Replacement.

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
    use dokime_gui::parameters;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("parameters.json");
    assert_eq!(parameters::load(&path).unwrap(), None);
    let values = [(
        "template".into(),
        [
            ("parameter".into(), parameters::Value::Legacy("12".into())),
            ("defined".into(), parameters::Value::Defined { raw: "115".into(), value_type: "int".into(), unit: Some("V".into()) }),
        ]
        .into(),
    )]
    .into();
    parameters::save(&path, &values).unwrap();
    assert_eq!(parameters::load(&path).unwrap(), Some(values));
    let saved = fs::read(&path).unwrap();
    let invalid = [("template".into(), [("parameter".into(), parameters::Value::Legacy("x".repeat(8193)))].into())].into();
    assert!(parameters::save(&path, &invalid).is_err());
    assert_eq!(fs::read(&path).unwrap(), saved);
    parameters::save(&path, &parameters::Values::new()).unwrap();
    assert_eq!(parameters::load(&path).unwrap(), Some(parameters::Values::new()));
    fs::write(&path, b"broken").unwrap();
    assert!(parameters::load(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"broken");
    fs::write(&path, vec![b' '; 1024 * 1024 + 1]).unwrap();
    assert!(parameters::load(&path).is_err());
}
