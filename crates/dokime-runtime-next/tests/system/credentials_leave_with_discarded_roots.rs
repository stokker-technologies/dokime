/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Discarded Story Roots Remove Their Protected Credentials

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, prost::Message)]
struct AccessPointer {
    #[prost(string, tag = "5")]
    key_slot: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn credentials_leave_with_discarded_roots() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        for fails in [false, true] {
            // Start Through the Product Journey and Read Its Published Credential by Name
            let root = super::support::tempdir()?;
            let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &[]).await?;
            let store = dokime_storage::Store::open(root.path())?;
            let pointer: AccessPointer = store.administration("access")?.ok_or("Read Access Credential: Missing Publication")?;
            let host_id = std::fs::read_to_string(root.path().join("backend-id"))?;
            let entry = keyring::Entry::new("dokime.access", &format!("{host_id}:{}", pointer.key_slot))?;
            assert!(!entry.get_password()?.is_empty());
            drop(store);
            process.close().await?;

            // Finish or Unwind the Story Before Discarding Its Root
            if fails {
                let failure = std::panic::catch_unwind(move || {
                    let _root = root;
                    panic!("Simulated Story Failure");
                });
                assert!(failure.is_err());
            } else {
                drop(root);
            }

            // The Created Name Is No Longer Readable
            assert!(matches!(entry.get_password(), Err(keyring::Error::NoEntry)));
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
