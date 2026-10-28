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
use harmos_packaging::bundle::{Bundle, Executable, Facility, Identity, Limits, Source};
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn bundle(package: &str, owner: &str, version: &str) -> Vec<u8> {
    let identity = Identity {
        name: package.into(),
        version: version.parse().unwrap(),
        description: "A fixture.".into(),
        package: Default::default(),
    };
    let declaration =
        serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":version,"description":"A fixture.","config":"()"});
    let policy = harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(120),
        attempt_timeout: std::time::Duration::from_secs(120),
        retries: harmos::operations::RetryPolicy::Never,
    };
    // Inert artifact metadata names each lifecycle action's identity scheme rather than its fingerprint.
    let lifecycle = serde_json::json!([
        {"action": "initialize", "identity": "harmos.operation.v1", "policy": policy},
        {"action": "shutdown", "identity": "harmos.operation.v1", "policy": policy},
    ]);
    let contracts =
        serde_json::json!({"kind":"contracts","facility":"sidecar","owner":owner,"provides":[],"requires":[],"lifecycle":lifecycle});
    let restart = serde_json::json!({"kind":"restart","facility":"sidecar","owner":owner,"policy":{"credit":3,"sustained_health_ms":30000,"backoff_base_ms":250,"backoff_cap_ms":10000}});
    let publications = serde_json::json!({"kind":"publications","facility":"sidecar","owner":owner,"streams":[]});
    let observers = serde_json::json!({"kind":"observers","facility":"sidecar","owner":owner,"observers":[]});
    let executable = format!(
        "inert{prefix}{declaration}\0{prefix}{contracts}\0{prefix}{restart}\0{prefix}{publications}\0{prefix}{observers}\0",
        prefix = std::str::from_utf8(harmos::artifacts::PREFIX).unwrap()
    );
    Bundle::build(
        identity,
        &[Source {
            path: "bin/fixture",
            bytes: executable.as_bytes(),
            executable: Some(Executable {
                facility: Facility::Sidecar,
                target: env!("DOKIME_NATIVE_TARGET").into(),
                registrations: vec![],
            }),
        }],
        Limits::default(),
    )
    .unwrap()
    .bytes()
    .to_vec()
}

pub fn obc_plugin_bundle() -> std::path::PathBuf {
    let profile = std::env::var("STKR_TEST_PROFILE").unwrap_or_else(|_| "dev".into());
    assert!(matches!(profile.as_str(), "dev" | "release"));
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect("nextest sets the manifest directory");
    let workspace = std::path::PathBuf::from(manifest)
        .ancestors()
        .find(|directory| {
            let lockfile = directory.join("Cargo.lock");
            lockfile.is_file()
        })
        .expect("the workspace root holds Cargo.lock")
        .to_owned();
    let target = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    let path = workspace.join("target/build").join(profile).join(target).join("plugins/obc-plugin.dokime");
    assert!(path.is_file(), "missing built artifact {}; run `mise run build`", path.display());
    path
}

/* === Private Functions === */

/* === Tests === */
