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

#[tokio::test]
async fn credentials_leave_with_discarded_roots() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        for fails in [false, true] {
            // Start Through the Product Journey and Read Its Published Credential by Name
            let root = super::support::tempdir()?;
            let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &[]).await?;
            let mut secrets = dokime::v1::secrets_client::SecretsClient::connect(process.url.clone()).await?;
            let initial = secrets.list(dokime::v1::SecretsListRequest { ..Default::default() }).await?.into_inner();
            let value = uuid::Uuid::now_v7().to_string();
            secrets
                .set(dokime::v1::SecretsSetRequest {
                    name: "story.access".into(),
                    value: Some(dokime::v1::SecretWritable {
                        metadata: Some(dokime::v1::Metadata {
                            identifier: "story.access".into(),
                            label: "Story Access".into(),
                            ..Default::default()
                        }),
                        value: Some(value.clone()),
                    }),
                    expected_revision: initial.generation.to_be_bytes().to_vec(),
                    request_id: "credential".into(),
                })
                .await?;
            let store = dokime_storage::Store::open(root.path())?;
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
            let store = dokime_storage::Store::open(&nested)?;
            let pointer: AccessPointer = store.administration("access")?.ok_or("Read Nested Credential: Missing Publication")?;
            let host_id = std::fs::read_to_string(nested.join("backend-id"))?;
            let nested_entry = keyring::Entry::new("dokime.access", &format!("{host_id}:{}", pointer.key_slot))?;
            assert!(!nested_entry.get_password()?.is_empty());
            drop(store);
            child.close().await?;
            process.close().await?;
            drop(dokime_storage::Store::open(&root.path().join("storage-only"))?);

            if fails {
                // The Durable Write Intent Also Owns Cleanup Without a Published Pointer.
                let store = dokime_storage::Store::open(root.path())?;
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
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
