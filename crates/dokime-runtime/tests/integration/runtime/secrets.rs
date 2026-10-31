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
use harmos::operations::{InvocationError, Visibility};
use harmos::plugin::{
    PluginArtifact, PluginBackend, PluginContributions, PluginDefinition, PluginDefinitions, PluginEndpoint, PluginLifecycle,
    PluginLifecycleAuthority, PluginManager, PluginTransition, Router,
};

// Local Imports
use dokime_domain::model::secrets::catalog::{Metadata, Value};
use dokime_runtime::operations::secrets;
use dokime_runtime::{Runtime, app::Lanes};

/* === Constants === */

const OWNERS: [&str; 2] = ["plugin_alpha", "plugin_beta"];

/* === Macros === */

/* === Type Aliases === */

type Operations = harmos::operations::OperationManager<Lanes>;

/* === Structs === */

#[harmos::operation(response = Value, id = "fixture.secrets.read", policy = dokime_operations::ORDINARY, operations = [secrets::Lookup])]
#[derive(PartialEq)]
struct ReadSecret {
    identifier: String,
}

// Stages `ReadSecret` under each plugin owner's managed activation scope.
struct Owners(Operations);

#[derive(Default)]
struct Staged {
    endpoint: Option<PluginEndpoint>,
    registration: Option<harmos::operations::PluginRegistration>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl harmos::Operation for ReadSecret {
    type Error = InvocationError;

    async fn run(self, ctx: harmos::Context<'_, Self>) -> Result<harmos::Response<Value>, InvocationError> {
        ctx.operation(secrets::Lookup { identifier: self.identifier }).await
    }
}

impl PluginLifecycle for Owners {
    type Guest = ();
    type Sidecar = ();
    type Embedded = Staged;

    fn preflight(&mut self, candidate: &PluginDefinitions) -> harmos::Result<()> {
        self.0.validate_requirements(candidate)
    }

    fn prepare(&mut self, _: &PluginDefinition) -> harmos::Result<PluginBackend<(), (), Staged>> {
        Ok(PluginBackend::Embedded(Staged::default()))
    }

    async fn fence(&mut self, _: PluginTransition, _: &[PluginDefinition]) -> harmos::Result<()> {
        Ok(())
    }

    async fn start(
        &mut self, definition: &PluginDefinition, router: &Router, _: &PluginLifecycleAuthority<'_>,
        backend: &mut PluginBackend<(), (), Staged>,
    ) -> harmos::Result<()> {
        let PluginBackend::Embedded(staged) = backend else { unreachable!() };
        let endpoint = router.enter(&definition.id).map_err(|source| harmos::Error::TaskFailed {
            domain: "fixture owner",
            source: Arc::new(source),
        })?;
        let scope = endpoint.scope().cloned().expect("a managed owner has an activation scope");
        staged.endpoint = Some(endpoint);
        staged.registration = Some(self.0.stage_plugin(&scope, |bindings| bindings.operations::<SecretOperations>(Lanes::Open))?);
        Ok(())
    }

    async fn stop(
        _: &PluginDefinition, _: &PluginLifecycleAuthority<'_>, backend: &mut PluginBackend<(), (), Staged>,
    ) -> harmos::Result<()> {
        let PluginBackend::Embedded(staged) = backend else { unreachable!() };
        if let Some(registration) = staged.registration.as_mut() {
            registration.shutdown().await?;
        }
        staged.registration.take();
        staged.endpoint.take();
        Ok(())
    }
}

fn owners() -> PluginDefinitions {
    let definitions = OWNERS.map(|owner| PluginDefinition {
        id: owner.into(),
        version: "0.1.0".parse().unwrap(),
        artifact: PluginArtifact::Embedded { revision: "fixture".into() },
        configuration: b"null".to_vec(),
        contributions: PluginContributions {
            operations: vec![<ReadSecret as harmos::OperationDeclaration>::definition()],
            wire_contracts: vec![harmos::operations::WireDeclaration::of::<ReadSecret>().contract().unwrap()],
            // Each owner requires the host lookup it calls, the way an authored plugin declares it.
            required_contracts: Some(
                harmos::domain::inventory::contracts::Required::from_tree(&[harmos::domain::catalog::Node::Leaf(
                    harmos::domain::plugin::Requirement::of::<secrets::Lookup>(&[]),
                )])
                .unwrap(),
            ),
            ..Default::default()
        },
        requires: vec![],
    });
    PluginDefinitions::new(definitions, []).unwrap()
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn two_plugin_owners_resolve_shared_secret_lifecycle() {
    let root = tempfile::tempdir().unwrap();
    let config = crate::service_support::config(root.path());
    let mut runtime = Runtime::start(config).await.unwrap();
    let handle = runtime.handle();
    let operations = handle.operations();
    assert_eq!(operations.definition("dokime.secrets.lookup").unwrap().visibility, Visibility::Public);
    assert_eq!(operations.definition("dokime.secrets.put").unwrap().visibility, Visibility::Host);
    let mut lifecycle = Owners(operations.clone());
    let mut plugins = PluginManager::new(owners()).unwrap();
    plugins.initialize(&mut lifecycle).await.unwrap();

    let metadata = Metadata {
        identifier: "shared.bench".into(),
        label: "Shared Bench".into(),
        description: None,
    };
    for (generation, value) in [(0, "first-test-value"), (1, "replacement-test-value")] {
        let written = operations
            .call::<secrets::Put>(secrets::Put {
                metadata: metadata.clone(),
                value: Some(value.into()),
                generation,
            })
            .await
            .unwrap();
        assert_eq!(written.generation, generation + 1);
        for owner in OWNERS {
            let reading =
                operations.submit_from::<ReadSecret>(owner, ReadSecret { identifier: metadata.identifier.clone() }).unwrap().await.unwrap();
            assert_eq!(reading.value.as_deref(), Some(value));
            assert!(!format!("{reading:?}").contains(value));
        }
    }

    operations
        .call::<secrets::Remove>(secrets::Remove {
            identifiers: vec![metadata.identifier.clone()],
            generation: 2,
        })
        .await
        .unwrap();
    for owner in OWNERS {
        let result = operations.submit_from::<ReadSecret>(owner, ReadSecret { identifier: metadata.identifier.clone() }).unwrap().await;
        assert!(result.is_err());
    }
    let connection =
        rusqlite::Connection::open_with_flags(handle.storage_root().join("local.sqlite"), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap();
    let body: Vec<u8> = connection.query_row("SELECT body FROM application_metadata WHERE id=1", [], |row| row.get(0)).unwrap();
    let settings = String::from_utf8(body).unwrap();
    assert!(!settings.contains("first-test-value"));
    assert!(!settings.contains("replacement-test-value"));
    plugins.shutdown(&mut lifecycle).await.unwrap();
    drop(handle);
    runtime.shutdown().await.unwrap();
}

#[harmos::catalog]
enum SecretOperations {
    Read(ReadSecret),
}
