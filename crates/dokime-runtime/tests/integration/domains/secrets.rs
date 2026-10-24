/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Real Secret Transfers Authenticate Before Publishing Settings And Credentials.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::any::Any;
use std::collections::{BTreeMap, HashMap};
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicIsize, Ordering},
};

// Third Party Imports
use keyring_core::api::{CredentialApi, CredentialStoreApi};

// Local Imports
use dokime_domain::model::secrets::{
    catalog::{Metadata, Snapshot},
    transfer::{Action, Choice, MAX_CIPHERTEXT_BYTES},
};
use dokime_domain::operations::secrets as wire;
use dokime_runtime::{Runtime, RuntimeConfig, RuntimeHandle};

/* === Constants === */

const PASSWORD: &str = "correct horse battery staple";

// Synthetic v1 Documents Encrypted By age 0.12.1 With Scrypt Work Factor 18.
// VALID Contains alpha/beta Export Values; INVALID Contains The Rejected ../outside Name.
const VALID: &[u8] = include_bytes!("../../fixtures/secrets/valid.age");
const INVALID: &[u8] = include_bytes!("../../fixtures/secrets/invalid.age");

/* === Macros === */

/* === Type Aliases === */

type Release = Arc<(Mutex<bool>, Condvar)>;

/* === Structs === */

struct Store {
    id: String,
    state: Arc<StoreState>,
}

struct StoreState {
    values: Mutex<BTreeMap<(String, String), Vec<u8>>>,
    remaining: AtomicIsize,
    read: Mutex<Option<ReadBlock>>,
}

struct Credential {
    state: Arc<StoreState>,
    key: (String, String),
}

struct ReadBlock {
    entered: tokio::sync::oneshot::Sender<()>,
    release: Release,
}

struct ReadHold(Release);

struct Fixture {
    root: tempfile::TempDir,
    runtime: Runtime,
    handle: RuntimeHandle,
    store: Arc<Store>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl CredentialStoreApi for Store {
    fn vendor(&self) -> String {
        "Dokime Integration Tests".into()
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn persistence(&self) -> keyring_core::CredentialPersistence {
        keyring_core::CredentialPersistence::ProcessOnly
    }

    fn build(&self, service: &str, user: &str, _: Option<&HashMap<&str, &str>>) -> keyring_core::Result<keyring_core::Entry> {
        let credential = Credential { state: Arc::clone(&self.state), key: (service.into(), user.into()) };
        let credential = Arc::new(credential);
        Ok(keyring_core::Entry::new_with_credential(credential))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CredentialApi for Credential {
    fn set_secret(&self, value: &[u8]) -> keyring_core::Result<()> {
        if self.state.remaining.load(Ordering::SeqCst) == 0 {
            return Err(keyring_core::Error::NoEntry);
        }
        self.state.remaining.fetch_sub(1, Ordering::SeqCst);
        self.state.values.lock().unwrap().insert(self.key.clone(), value.to_vec());
        Ok(())
    }

    fn get_secret(&self) -> keyring_core::Result<Vec<u8>> {
        let value = self.state.values.lock().unwrap().get(&self.key).cloned().ok_or(keyring_core::Error::NoEntry)?;
        let block = self.state.read.lock().unwrap().take();
        if let Some(block) = block {
            let _ = block.entered.send(());
            let (released, changed) = &*block.release;
            let guard = released.lock().unwrap();
            let released = changed.wait_while(guard, |released| !*released).unwrap();
            drop(released);
        }
        Ok(value)
    }

    fn delete_credential(&self) -> keyring_core::Result<()> {
        let removed = self.state.values.lock().unwrap().remove(&self.key);
        removed.map(|_| ()).ok_or(keyring_core::Error::NoEntry)
    }

    fn get_credential(&self) -> keyring_core::Result<Option<Arc<keyring_core::Credential>>> {
        self.get_secret()?;
        Ok(None)
    }

    fn get_specifiers(&self) -> Option<(String, String)> {
        Some(self.key.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for ReadHold {
    fn drop(&mut self) {
        let (released, changed) = &*self.0;
        *released.lock().unwrap() = true;
        changed.notify_all();
    }
}

impl Fixture {
    async fn start() -> Self {
        for archive in [VALID, INVALID] {
            assert!(archive.starts_with(b"age-encryption.org/v1\n"));
            assert!(archive.split(|byte| *byte == b'\n').nth(1).unwrap().ends_with(b" 18"));
        }

        let root = tempfile::tempdir().unwrap();
        let state = StoreState { values: Mutex::default(), remaining: AtomicIsize::new(-1), read: Mutex::default() };
        let store = Arc::new(Store { id: root.path().display().to_string(), state: Arc::new(state) });
        let metadata = crate::service_support::metadata("secret.fixture", "Secret Host");
        let config = RuntimeConfig::new(root.path(), metadata).with_credential_store(store.clone());
        let runtime = Runtime::start(config).await.unwrap();
        let handle = runtime.handle();
        let fixture = Self { root, runtime, handle, store };

        let operations = fixture.handle.operations();
        operations.call(put("alpha", "old-alpha", 0)).await.unwrap();
        operations.call(put("beta", "retained-beta", 1)).await.unwrap();
        fixture
    }

    fn slots(&self) -> usize {
        self.store.state.values.lock().unwrap().len()
    }

    async fn catalog(&self) -> Snapshot {
        (*self.runtime.handle().operations().call(wire::List {}).await.unwrap()).clone()
    }

    async fn stop(mut self) {
        self.runtime.shutdown().await.unwrap();
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn put(name: &str, value: &str, generation: u64) -> wire::Put {
    let metadata = Metadata { name: name.into(), label: name.into(), description: None };
    wire::Put { metadata, value: Some(value.into()), generation }
}

fn export(generation: u64) -> wire::Export {
    wire::Export { names: vec!["alpha".into(), "beta".into()], password: PASSWORD.into(), generation }
}

fn import(ciphertext: &[u8], generation: u64) -> wire::Import {
    let choices = vec![Choice { name: "alpha".into(), action: Action::Replace }, Choice { name: "beta".into(), action: Action::Skip }];
    wire::Import { ciphertext: ciphertext.into(), password: PASSWORD.into(), choices, generation }
}

/* === Tests === */

#[tokio::test]
async fn export_preview() {
    let source = Fixture::start().await;
    let operations = source.handle.operations();
    operations.call(put("alpha", "exported-alpha-value", 2)).await.unwrap();
    operations.call(put("beta", "exported-beta-value", 3)).await.unwrap();
    let archive = operations.call(export(4)).await.unwrap();
    assert!(!archive.ciphertext.windows(20).any(|bytes| bytes == b"exported-alpha-value"));

    let destination = Fixture::start().await;
    let preview = destination
        .runtime
        .handle()
        .operations()
        .call(wire::Preview { ciphertext: archive.ciphertext.clone(), password: PASSWORD.into() })
        .await
        .unwrap();
    assert_eq!(preview.generation, 2);
    assert_eq!(preview.host_id, destination.catalog().await.host_id);
    assert_ne!(preview.host_id, source.catalog().await.host_id);
    assert!(preview.entries.iter().all(|entry| entry.exists));
    assert!(!serde_json::to_string(&*preview).unwrap().contains("exported-alpha-value"));

    source.stop().await;
    destination.stop().await;
}

#[tokio::test]
async fn import_choices() {
    let destination = Fixture::start().await;
    let operations = destination.handle.operations();
    operations.call(import(VALID, 2)).await.unwrap();
    let alpha = operations.call(wire::Lookup { name: "alpha".into() }).await.unwrap();
    let beta = operations.call(wire::Lookup { name: "beta".into() }).await.unwrap();
    assert_eq!(alpha.value.as_deref(), Some("exported-alpha-value"));
    assert_eq!(beta.value.as_deref(), Some("retained-beta"));
    destination.stop().await;
}

#[tokio::test]
async fn import_replaces_all() {
    let destination = Fixture::start().await;
    let operations = destination.handle.operations();
    let mut request = import(VALID, 2);
    request.choices[1].action = Action::Replace;
    operations.call(request).await.unwrap();

    let alpha = operations.call(wire::Lookup { name: "alpha".into() }).await.unwrap();
    let beta = operations.call(wire::Lookup { name: "beta".into() }).await.unwrap();
    assert_eq!(alpha.value.as_deref(), Some("exported-alpha-value"));
    assert_eq!(beta.value.as_deref(), Some("exported-beta-value"));
    destination.stop().await;
}

#[tokio::test]
async fn wrong_password() {
    let destination = Fixture::start().await;
    let committed = destination.catalog().await;
    let slots = destination.slots();

    let mut request = import(VALID, 2);
    request.password = "wrong password".into();
    assert!(destination.runtime.handle().operations().call(request).await.is_err());
    assert_eq!(destination.catalog().await, committed);
    assert_eq!(destination.slots(), slots);
    destination.stop().await;
}

#[tokio::test]
async fn corrupted_archive() {
    let destination = Fixture::start().await;
    let committed = destination.catalog().await;
    let slots = destination.slots();

    let mut ciphertext = VALID.to_vec();
    *ciphertext.last_mut().unwrap() ^= 1;
    assert!(destination.runtime.handle().operations().call(import(&ciphertext, 2)).await.is_err());
    assert_eq!(destination.catalog().await, committed);
    assert_eq!(destination.slots(), slots);
    destination.stop().await;
}

#[tokio::test]
async fn appended_archive() {
    let destination = Fixture::start().await;
    let committed = destination.catalog().await;
    let slots = destination.slots();

    let mut ciphertext = VALID.to_vec();
    ciphertext.push(0);
    assert!(destination.runtime.handle().operations().call(import(&ciphertext, 2)).await.is_err());
    assert_eq!(destination.catalog().await, committed);
    assert_eq!(destination.slots(), slots);
    destination.stop().await;
}

#[tokio::test]
async fn truncated_archive() {
    let destination = Fixture::start().await;
    let committed = destination.catalog().await;
    let slots = destination.slots();
    let ciphertext = &VALID[..VALID.len() - 1];
    assert!(destination.runtime.handle().operations().call(import(ciphertext, 2)).await.is_err());
    assert_eq!(destination.catalog().await, committed);
    assert_eq!(destination.slots(), slots);
    destination.stop().await;
}

#[tokio::test]
async fn invalid_document() {
    let destination = Fixture::start().await;
    let committed = destination.catalog().await;
    let slots = destination.slots();

    let mut request = import(INVALID, 2);
    request.choices = vec![Choice { name: "../outside".into(), action: Action::Replace }];
    assert!(destination.runtime.handle().operations().call(request).await.is_err());
    assert_eq!(destination.catalog().await, committed);
    assert_eq!(destination.slots(), slots);
    destination.stop().await;
}

#[tokio::test]
async fn oversized_archive() {
    let destination = Fixture::start().await;
    let committed = destination.catalog().await;
    let slots = destination.slots();
    let request = wire::Preview { ciphertext: vec![0; MAX_CIPHERTEXT_BYTES + 1], password: PASSWORD.into() };
    assert!(destination.runtime.handle().operations().call(request).await.is_err());
    assert_eq!(destination.catalog().await, committed);
    assert_eq!(destination.slots(), slots);
    destination.stop().await;
}

#[tokio::test]
async fn stale_generation() {
    let destination = Fixture::start().await;
    let committed = destination.catalog().await;
    let slots = destination.slots();
    assert!(destination.runtime.handle().operations().call(import(VALID, 1)).await.is_err());
    assert_eq!(destination.catalog().await, committed);
    assert_eq!(destination.slots(), slots);
    destination.stop().await;
}

#[tokio::test]
async fn write_rollback() {
    let destination = Fixture::start().await;
    let committed = destination.catalog().await;
    let slots = destination.slots();

    destination.store.state.remaining.store(1, Ordering::SeqCst);
    let mut request = import(VALID, 2);
    request.choices[1].action = Action::Replace;
    assert!(destination.runtime.handle().operations().call(request).await.is_err());
    assert_eq!(destination.catalog().await, committed);
    assert_eq!(destination.slots(), slots);

    let beta = destination.runtime.handle().operations().call(wire::Lookup { name: "beta".into() }).await.unwrap();
    assert_eq!(beta.value.as_deref(), Some("retained-beta"));
    let alpha = destination.runtime.handle().operations().call(wire::Lookup { name: "alpha".into() }).await.unwrap();
    assert_eq!(alpha.value.as_deref(), Some("old-alpha"));
    destination.stop().await;
}

#[tokio::test]
async fn settings_rollback() {
    let destination = Fixture::start().await;
    let operations = destination.handle.operations();
    operations.call(put("alpha", "exported-alpha-value", 2)).await.unwrap();
    let committed = destination.catalog().await;
    let slots = destination.slots();
    let settings_path = destination.root.path().join("application/settings.json");
    let saved = std::fs::read(&settings_path).unwrap();
    assert!(!String::from_utf8_lossy(&saved).contains("exported-alpha-value"));

    std::fs::write(&settings_path, b"invalid settings").unwrap();
    assert!(operations.call(put("alpha", "unconfirmed-value", 3)).await.is_err());
    std::fs::write(&settings_path, &saved).unwrap();
    assert_eq!(destination.catalog().await, committed);

    let alpha = operations.call(wire::Lookup { name: "alpha".into() }).await.unwrap();
    assert_eq!(alpha.value.as_deref(), Some("exported-alpha-value"));
    assert_eq!(destination.slots(), slots + 1);
    destination.stop().await;
}

#[tokio::test]
async fn transfer_exclusivity() {
    let source = Fixture::start().await;
    let (entered, waiting) = tokio::sync::oneshot::channel();
    let release = Arc::new((Mutex::new(false), Condvar::new()));
    let hold = ReadHold(Arc::clone(&release));
    *source.store.state.read.lock().unwrap() = Some(ReadBlock { entered, release });

    let operations = source.handle.operations();
    let first = operations.submit(export(2)).unwrap();
    waiting.await.unwrap();
    assert!(operations.call(export(2)).await.is_err());
    drop(hold);
    first.join().await.unwrap();
    source.stop().await;
}
