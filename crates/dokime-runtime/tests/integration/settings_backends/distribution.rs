/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Installed Identity Resolution

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_runtime::paths::{ApplicationPaths, DEFAULT_APPLICATION_ID, distribution_identity};

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

#[dokime_tracing::trace_test]
#[test]
fn distribution_file() {
    let root = tempfile::tempdir().unwrap();
    let executable = root.path().join("dokime");
    let file = root.path().join("distribution.json");
    assert_eq!(distribution_identity(&executable, DEFAULT_APPLICATION_ID).unwrap().identifier, DEFAULT_APPLICATION_ID);
    std::fs::write(&file, br#"{"schema":1,"identifier":"com.example.testhub"}"#).unwrap();
    let identity = distribution_identity(&executable, DEFAULT_APPLICATION_ID).unwrap();
    assert_eq!(identity.identifier, "com.example.testhub");
    let application = root.path().join("application");
    let paths = ApplicationPaths::resolve_for(&executable, DEFAULT_APPLICATION_ID, Some(application.clone())).unwrap();
    assert_eq!(paths.root, application);
    assert_eq!(paths.logs(), application.join("logs"));
    for bytes in [br#"{"schema":2,"identifier":"com.example.testhub"}"#.as_slice(), br#"{"schema":1,"identifier":"../escape"}"#, b"broken"]
    {
        std::fs::write(&file, bytes).unwrap();
        assert!(ApplicationPaths::resolve_for(&executable, DEFAULT_APPLICATION_ID, Some(application.clone())).is_err());
    }
}
