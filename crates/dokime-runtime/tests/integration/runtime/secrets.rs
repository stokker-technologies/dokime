/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Two Plugin-Owned Operations Resolve The Same Live Host Secret.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use harmos::operations::{Context, Operation, Response, Visibility};

// Local Imports
use dokime_domain::{
    model::secrets::catalog::{Metadata, Value},
    operations::secrets,
};
use dokime_runtime::{Runtime, app::Lanes};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::operation(response = Value, id = "fixture.secrets.read", policy = dokime_domain::operations::ORDINARY)]
#[derive(Clone, PartialEq)]
struct ReadSecret {
    identifier: String,
}

struct Reader;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for Reader {
    type Declaration = ReadSecret;
    type Resources = ();
    type Error = harmos::Error;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(_: Arc<()>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self)
    }

    async fn run<'a>(&mut self, ctx: Self::Context<'a>, request: &mut ReadSecret, _: &mut ()) -> Result<Response<Value>, Self::Error> {
        ctx.call(secrets::Lookup { identifier: request.identifier.clone() }).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn two_plugin_owners_resolve_shared_secret_lifecycle() {
    let root = tempfile::tempdir().unwrap();
    let config = crate::service_support::config(root.path());
    let mut runtime = Runtime::start(config).await.unwrap();
    let handle = runtime.handle();
    let operations = handle.operations();
    assert_eq!(operations.definition("dokime.secrets.lookup").unwrap().visibility, Visibility::Shared);
    assert_eq!(operations.definition("dokime.secrets.put").unwrap().visibility, Visibility::Host);
    for owner in ["plugin.alpha", "plugin.beta"] {
        operations.register_operation::<Reader>(owner, Lanes::Open, Arc::new(())).unwrap();
    }

    let metadata = Metadata { identifier: "shared.bench".into(), label: "Shared Bench".into(), description: None };
    for (generation, value) in [(0, "first-test-value"), (1, "replacement-test-value")] {
        let written = operations.call(secrets::Put { metadata: metadata.clone(), value: Some(value.into()), generation }).await.unwrap();
        assert_eq!(written.generation, generation + 1);
        for owner in ["plugin.alpha", "plugin.beta"] {
            let reading = operations.submit_from(owner, ReadSecret { identifier: metadata.identifier.clone() }).unwrap().await.unwrap();
            assert_eq!(reading.value.as_deref(), Some(value));
            assert!(!format!("{reading:?}").contains(value));
        }
    }

    operations.call(secrets::Remove { identifiers: vec![metadata.identifier.clone()], generation: 2 }).await.unwrap();
    for owner in ["plugin.alpha", "plugin.beta"] {
        let result = operations.submit_from(owner, ReadSecret { identifier: metadata.identifier.clone() }).unwrap().await;
        assert!(result.is_err());
    }
    let connection =
        rusqlite::Connection::open_with_flags(handle.storage_root().join("local.sqlite"), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap();
    let body: Vec<u8> = connection.query_row("SELECT body FROM application_metadata WHERE id=1", [], |row| row.get(0)).unwrap();
    let settings = String::from_utf8(body).unwrap();
    assert!(!settings.contains("first-test-value"));
    assert!(!settings.contains("replacement-test-value"));
    drop(handle);
    runtime.shutdown().await.unwrap();
}
