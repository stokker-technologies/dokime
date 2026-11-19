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
use std::{
    error::Error,
    fs, io, panic,
    path::Path,
    sync::{Arc, Barrier, Mutex},
    thread,
};

// Third Party Imports
use prost::Message;
use tonic::transport::Endpoint;
use uuid::Uuid;

// Local Imports
use dokime::Metadata;
use dokime::SecretWritable;
use dokime_client_internal::Client;
use dokime_client_internal::SecretsSetOptions;
use dokime_runtime::credentials::Entry;
use dokime_storage::Store;
/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Message)]
struct AccessPointer {
    #[prost(string, tag = "5")]
    key_slot: String,
}

#[derive(Clone, PartialEq, Message)]
struct SecretPointer {
    #[prost(message, repeated, tag = "2")]
    entries: Vec<SecretSlot>,
}

#[derive(Clone, PartialEq, Message)]
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
fn concurrent_hosts_remove_native_credentials_across_processes() {
    let barrier = Arc::new(Barrier::new(2));
    let names = thread::scope(|scope| {
        let workers: Vec<_> = (0..2)
            .map(|_| {
                let barrier = barrier.clone();
                scope.spawn(move || {
                    super::support::run(Box::pin(async move {
                        let root = super::support::tempdir().unwrap();
                        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &[])
                            .await
                            .unwrap();
                        let store = Store::open(root.path()).unwrap();
                        let pointer: AccessPointer = store.administration("access").unwrap().unwrap();
                        let name = format!("{}:{}", store.identity(), pointer.key_slot);
                        assert!(
                            !Entry::new(&super::support::namespace().service("access"), &name).unwrap().get_password().unwrap().is_empty()
                        );
                        tokio::task::spawn_blocking(move || barrier.wait()).await.unwrap();
                        process.close().await.unwrap();
                        drop(store);
                        drop(root);
                        // Read the Native Store on This Thread After the Child Process Exits.
                        assert!(matches!(
                            Entry::new(&super::support::namespace().service("access"), &name).unwrap().get_password(),
                            Err(keyring::Error::NoEntry)
                        ));
                        name
                    }))
                })
            })
            .collect();
        workers.into_iter().map(|worker| worker.join().unwrap()).collect::<Vec<_>>()
    });
    for name in names {
        assert!(matches!(
            Entry::new(&super::support::namespace().service("access"), &name).unwrap().get_password(),
            Err(keyring::Error::NoEntry)
        ));
    }
}

#[test]
fn failed_live_story_removes_its_access_credential() {
    let created = Arc::new(Mutex::new(None));
    let recorded = created.clone();
    let failure = panic::catch_unwind(move || {
        super::support::run(Box::pin(async {
            let root = super::support::tempdir().unwrap();
            let _process = super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &[]).await.unwrap();
            let store = Store::open(root.path()).unwrap();
            let pointer: AccessPointer = store.administration("access").unwrap().unwrap();
            let host_id = fs::read_to_string(root.path().join("backend-id")).unwrap();
            let name = format!("{host_id}:{}", pointer.key_slot);
            let credential = Entry::new(&super::support::namespace().service("access"), &name).unwrap();
            assert!(!credential.get_password().unwrap().is_empty());
            *recorded.lock().unwrap() = Some((name, root.path().to_owned()));
            panic!("Simulated Failure With a Live Host");
        }))
    });
    assert!(failure.is_err());
    let (name, root) = created.lock().unwrap().take().expect("The Story Created Its Credential Before Failing");
    assert!(matches!(
        Entry::new(&super::support::namespace().service("access"), &name).unwrap().get_password(),
        Err(keyring::Error::NoEntry)
    ));
    assert!(!root.exists());
}

#[test]
fn credentials_leave_with_discarded_roots() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
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
                let value = Uuid::now_v7().to_string();
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
                let host_id = fs::read_to_string(root.path().join("backend-id"))?;
                let access_account = format!("{host_id}:{}", pointer.key_slot);
                let entry = Entry::new(&super::support::namespace().service("access"), &access_account)?;
                assert!(!entry.get_password()?.is_empty());
                let pointer: SecretPointer = store.administration("secrets")?.ok_or("Read Secret Credential: Missing Publication")?;
                let secret_account = format!("{host_id}:{}", pointer.entries.first().ok_or("Read Secret Credential: Missing Slot")?.slot);
                let secret = Entry::new(&super::support::namespace().service("shared-secrets"), &secret_account)?;
                assert_eq!(secret.get_password()?, value);
                drop(store);

                // A Disposable Root Can Also Own a Nested Backend Root.
                let nested = root.path().join("nested");
                let child = super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(&nested, &[]).await?;
                let store = Store::open(&nested)?;
                let pointer: AccessPointer = store.administration("access")?.ok_or("Read Nested Credential: Missing Publication")?;
                let nested_host_id = fs::read_to_string(nested.join("backend-id"))?;
                let nested_account = format!("{nested_host_id}:{}", pointer.key_slot);
                let nested_entry = Entry::new(&super::support::namespace().service("access"), &nested_account)?;
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
                    let failure = panic::catch_unwind(move || {
                        let _root = root;
                        panic!("Simulated Story Failure");
                    });
                    assert!(failure.is_err());
                } else {
                    drop(root);
                }

                // Fresh Native Reads Confirm Every Created Name Is Gone, Including Root Claims.
                for (owner, account) in [
                    ("access", access_account),
                    ("shared-secrets", secret_account),
                    ("access", nested_account),
                    ("root", host_id),
                    ("root", nested_host_id),
                ] {
                    let service = super::support::namespace().service(owner);
                    let credential = Entry::new(&service, &account)?;
                    assert!(matches!(credential.get_password(), Err(keyring::Error::NoEntry)));
                    eprintln!("Removed Test Credential: {account}.{service}");
                }
            }
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}

fn copy_folder(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            copy_folder(&entry.path(), &to.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), to.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[test]
fn copied_roots_never_remove_the_original_credentials() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // The Original Root Publishes an Access Key and a Stored Secret
            let root = super::support::tempdir()?;
            let server = dokime_runtime::services::serve(
                "127.0.0.1:0".parse()?,
                root.path(),
                super::support::settings(),
                super::support::service_settings(),
            )
            .await?;
            let secrets = Client::from_channel(
                Endpoint::from_shared(format!("http://{}", server.endpoint()))?.connect().await?,
                super::support::client_settings(),
            )?
            .secrets
            .clone();
            let initial = secrets.list().await?;
            let value = Uuid::now_v7().to_string();
            let request = SecretsSetOptions {
                request_id: Some("credential".into()),
                ..Default::default()
            };
            let writable = SecretWritable {
                metadata: Some(Metadata {
                    identifier: "story.copy".into(),
                    label: "Story Copy".into(),
                    ..Default::default()
                }),
                value: Some(value.clone()),
            };
            secrets.set_with("story.copy", writable, initial.generation.to_be_bytes().to_vec(), request).await?;
            server.close().await?;
            let store = Store::open(root.path())?;
            let host_id = store.identity().to_owned();
            let access: AccessPointer = store.administration("access")?.ok_or("Read Access Credential: Missing Publication")?;
            let access = Entry::new(&super::support::namespace().service("access"), &format!("{host_id}:{}", access.key_slot))?;
            let pointer: SecretPointer = store.administration("secrets")?.ok_or("Read Secret Credential: Missing Publication")?;
            let slot = &pointer.entries.first().ok_or("Read Secret Credential: Missing Slot")?.slot;
            let secret = Entry::new(&super::support::namespace().service("shared-secrets"), &format!("{host_id}:{slot}"))?;
            drop(store);

            // A Full Folder Copy Carries the Identity and Catalogs but Cannot Remove or Use Them
            let copy = super::support::tempdir()?;
            copy_folder(root.path(), copy.path())?;
            assert_eq!(fs::read_to_string(copy.path().join("backend-id"))?, host_id);
            assert!(dokime_runtime::services::remove_credentials(copy.path(), &super::support::namespace()).is_err());
            let opened = dokime_runtime::services::serve(
                "127.0.0.1:0".parse()?,
                copy.path(),
                super::support::settings(),
                super::support::service_settings(),
            )
            .await;
            assert!(opened.is_err());
            assert!(!access.get_password()?.is_empty());
            assert_eq!(secret.get_password()?, value);

            // The Copy Forgets Its Catalogs; the Original Still Owns and Removes Its Credentials
            dokime_runtime::services::detach_credentials(copy.path())?;
            drop(copy);
            assert_eq!(secret.get_password()?, value);
            dokime_runtime::services::remove_credentials(root.path(), &super::support::namespace())?;
            assert!(matches!(access.get_password(), Err(keyring::Error::NoEntry)));
            assert!(matches!(secret.get_password(), Err(keyring::Error::NoEntry)));
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}
