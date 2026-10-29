/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::fs;

#[test]
fn replacement_keeps_parameters() {
    use dokime_gui::parameters;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("parameters.json");
    assert_eq!(parameters::load(&path).unwrap(), None);
    let values = [(
        "template".into(),
        [
            (
                "parameter".into(),
                parameters::Value {
                    raw: "12".into(),
                    value_type: "int".into(),
                    unit: None,
                },
            ),
            (
                "defined".into(),
                parameters::Value {
                    raw: "115".into(),
                    value_type: "int".into(),
                    unit: Some("V".into()),
                },
            ),
        ]
        .into(),
    )]
    .into();
    parameters::save(&path, &values).unwrap();
    assert_eq!(parameters::load(&path).unwrap(), Some(values));
    let saved = fs::read(&path).unwrap();
    let invalid = [(
        "template".into(),
        [(
            "parameter".into(),
            parameters::Value {
                raw: "x".repeat(8193),
                value_type: "str".into(),
                unit: None,
            },
        )]
        .into(),
    )]
    .into();
    assert!(parameters::save(&path, &invalid).is_err());
    assert_eq!(fs::read(&path).unwrap(), saved);
    parameters::save(&path, &parameters::Values::new()).unwrap();
    assert_eq!(parameters::load(&path).unwrap(), Some(parameters::Values::new()));
    let unsupported = br#"{"template":{"parameter":"12"}}"#;
    fs::write(&path, unsupported).unwrap();
    let error = parameters::load(&path).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("unsupported remembered parameter format"));
    assert_eq!(fs::read(&path).unwrap(), unsupported);
    fs::write(&path, b"broken").unwrap();
    assert!(parameters::load(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"broken");
    fs::write(&path, vec![b' '; 1024 * 1024 + 1]).unwrap();
    assert!(parameters::load(&path).is_err());
}
