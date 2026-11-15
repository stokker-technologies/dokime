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
use dokime::Client;
use dokime::Metadata;
use dokime::SecretWritable;
use dokime::client::SecretsSetOptions;
use dokime_storage::Store;
use std::error::Error;
use tonic::transport::Endpoint;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, prost::Message)]
struct AccessPointer {
    #[prost(string, tag = "5")]
    key_slot: String,
}

#[derive(Clone, PartialEq, prost::Message)]
struct SecretPointer {
    #[prost(message, repeated, tag = "2")]
    entries: Vec<SecretSlot>,
}

#[derive(Clone, PartialEq, prost::Message)]
struct SecretSlot {
    #[prost(string, tag = "2")]
    slot: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn failed_live_story_removes_its_access_credential() {
    let created = std::sync::Arc::new(std::sync::Mutex::new(None));
    let recorded = created.clone();
    let failure = std::panic::catch_unwind(move || {
        super::support::run(async {
            let root = super::support::tempdir().unwrap();
            let _process = super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &[]).await.unwrap();
            let store = Store::open(root.path()).unwrap();
            let pointer: AccessPointer = store.administration("access").unwrap().unwrap();
            let host_id = std::fs::read_to_string(root.path().join("backend-id")).unwrap();
            let name = format!("{host_id}:{}", pointer.key_slot);
            let credential = keyring::Entry::new("dokime.access", &name).unwrap();
            assert!(!credential.get_password().unwrap().is_empty());
            *recorded.lock().unwrap() = Some((name, root.path().to_owned()));
            panic!("Simulated Failure With a Live Host");
        })
    });
    assert!(failure.is_err());
    let (name, root) = created.lock().unwrap().take().expect("The Story Created Its Credential Before Failing");
    assert!(matches!(keyring::Entry::new("dokime.access", &name).unwrap().get_password(), Err(keyring::Error::NoEntry)));
    assert!(!root.exists());
}

#[test]
fn credentials_leave_with_discarded_roots() -> Result<(), Box<dyn Error>> {
    super::support::run(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            for fails in [false, true] {
                // Start Through the Product Journey and Read Its Published Credential by Name
                let root = super::support::tempdir()?;
                let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &[]).await?;
                let secrets =
                    Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                        .secrets
                        .clone();
                let initial = secrets.list().await?;
                let value = uuid::Uuid::now_v7().to_string();
                secrets
                    .set_with(
                        "story.access",
                        SecretWritable {
                            metadata: Some(Metadata {
                                identifier: "story.access".into(),
                                label: "Story Access".into(),
                                ..Default::default()
                            }),
                            value: Some(value.clone()),
                        },
                        initial.generation.to_be_bytes().to_vec(),
                        SecretsSetOptions {
                            request_id: Some("credential".into()),
                            ..Default::default()
                        },
                    )
                    .await?;
                let store = Store::open(root.path())?;
                let pointer: AccessPointer = store.administration("access")?.ok_or("Read Access Credential: Missing Publication")?;
                let host_id = std::fs::read_to_string(root.path().join("backend-id"))?;
                let entry = keyring::Entry::new("dokime.access", &format!("{host_id}:{}", pointer.key_slot))?;
                assert!(!entry.get_password()?.is_empty());
                let pointer: SecretPointer = store.administration("secrets")?.ok_or("Read Secret Credential: Missing Publication")?;
                let secret = keyring::Entry::new(
                    "dokime.shared-secrets",
                    &format!("{host_id}:{}", pointer.entries.first().ok_or("Read Secret Credential: Missing Slot")?.slot),
                )?;
                assert_eq!(secret.get_password()?, value);
                drop(store);

                // A Disposable Root Can Also Own a Nested Backend Root.
                let nested = root.path().join("nested");
                let child = super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(&nested, &[]).await?;
                let store = Store::open(&nested)?;
                let pointer: AccessPointer = store.administration("access")?.ok_or("Read Nested Credential: Missing Publication")?;
                let host_id = std::fs::read_to_string(nested.join("backend-id"))?;
                let nested_entry = keyring::Entry::new("dokime.access", &format!("{host_id}:{}", pointer.key_slot))?;
                assert!(!nested_entry.get_password()?.is_empty());
                drop(store);
                child.close().await?;
                process.close().await?;
                drop(Store::open(&root.path().join("storage-only"))?);

                if fails {
                    // The Durable Write Intent Also Owns Cleanup Without a Published Pointer.
                    let store = Store::open(root.path())?;
                    store.delete_administration("access")?;
                    store.delete_administration("secrets")?;
                    assert!(!entry.get_password()?.is_empty());
                    assert!(!secret.get_password()?.is_empty());
                }

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
                assert!(matches!(secret.get_password(), Err(keyring::Error::NoEntry)));
                assert!(matches!(nested_entry.get_password(), Err(keyring::Error::NoEntry)));
            }
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    })
}
