/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Capability Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::error::Error as StdError;

// Third Party Imports

// Local Imports
use dokime_protocol::v1;
use dokime_runtime_next::Host;
use dokime_storage::{Execution, ExecutionArchive, ExecutionState, Store};
use prost::Message;

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

#[tokio::test]
async fn execution_capabilities() -> Result<(), Box<dyn StdError>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Inspect and Cancel an Active Plugin Run
        let root = super::support::tempdir()?;
        let host = Host::open(root.path(), super::support::settings())?;
        let entry = host.add_managed(&super::support::execution_fixture()?).await?;
        let origin = host.materialize(&entry, "execution.capture", super::support::parameters(29.0, 0, true)).await?;
        let cycle = host.start_cycle(vec![origin]).await?;
        let execution = cycle.executions[0].clone();
        host.next_prompt(&execution).await?;
        let active = host.capabilities(&execution)?;
        assert!(active.cancel && active.inspect);
        assert!(!active.start && !active.rerun && !active.export);
        assert!(host.rerun(&execution).await.is_err());
        host.cancel(&execution)?;
        cycle.wait().await?;
        let sealed = host.capabilities(&execution)?;
        assert!(sealed.rerun && sealed.inspect && sealed.export);
        assert!(!sealed.start && !sealed.cancel);
        assert!(host.cancel(&execution).unwrap_err().to_string().to_lowercase().contains("cancel"));
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(host.export_execution(&execution)?))?;
        let mut metadata = Vec::new();
        std::io::Read::read_to_end(&mut archive.by_name("evidence.pb")?, &mut metadata)?;
        let exported = ExecutionArchive::decode(metadata.as_slice())?;
        assert_eq!(exported.execution.as_ref().unwrap().identifier, execution);

        // Rerun Creates New Evidence and Preserves the Earlier Execution
        let before = host.execution(&execution)?;
        let cycle = host.rerun(&execution).await?;
        assert_ne!(cycle.executions[0], execution);
        let prompt = host.next_prompt(&cycle.executions[0]).await?;
        super::support::answer(&host, &cycle.executions[0], prompt)?;
        cycle.wait().await?;
        assert_eq!(host.execution(&execution)?, before);

        // Construct Import Evidence Directly Without Registering a Plugin
        let store = Store::open(root.path())?;
        store.create(&Execution {
            identifier: "unstarted".into(),
            cycle: "queued-cycle".into(),
            origin: before.execution.origin.clone(),
            catalog: before.execution.catalog.clone(),
            ..Default::default()
        })?;
        assert!(host.capabilities("unstarted")?.start);
        let cycle = host.start_execution("unstarted").await?;
        assert_eq!(cycle.executions, vec!["unstarted"]);
        let prompt = host.next_prompt("unstarted").await?;
        super::support::answer(&host, "unstarted", prompt)?;
        cycle.wait().await?;
        assert!(!host.capabilities("unstarted")?.start);
        let import = v1::Origin {
            kind: Some(v1::origin::Kind::Import(v1::Import {
                importer: "sample.importer".into(),
                processing: "Captured Measurements".into(),
                ..Default::default()
            })),
        };
        for identifier in ["imported", "draft", "closed-import"] {
            store.create(&Execution {
                identifier: identifier.into(),
                cycle: "import-cycle".into(),
                origin: Some(import.clone()),
                ..Default::default()
            })?;
        }
        store.close(
            "imported",
            v1::Termination::Completed,
            v1::ExecutionOutcome {
                attribution: Some(v1::execution_outcome::Attribution::Declared(v1::DeclaredOutcome {
                    status: v1::ResultStatus::Passed as i32,
                    importer: "sample.importer".into(),
                    ..Default::default()
                })),
            },
        )?;
        store.seal("imported")?;
        store.close(
            "closed-import",
            v1::Termination::Completed,
            v1::ExecutionOutcome {
                attribution: Some(v1::execution_outcome::Attribution::NotEvaluated(v1::UnevaluatedOutcome {
                    termination: v1::Termination::Completed as i32,
                    ..Default::default()
                })),
            },
        )?;
        let closed = store.read("closed-import")?;
        let capabilities = host.capabilities("closed-import")?;
        assert!(capabilities.inspect);
        assert!(!capabilities.start && !capabilities.rerun && !capabilities.cancel && !capabilities.export);
        let imported = host.capabilities("imported")?;
        assert!(imported.inspect && imported.export);
        assert!(!imported.start && !imported.rerun && !imported.cancel);
        assert!(host.start_cycle(vec![import]).await.is_err());
        assert!(host.start_execution("imported").await.is_err());
        assert!(host.rerun("imported").await.is_err());
        assert!(host.cancel("imported").is_err());
        let imported_bytes = host.export_execution("imported")?;
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&imported_bytes))?;
        let mut metadata = Vec::new();
        std::io::Read::read_to_end(&mut archive.by_name("evidence.pb")?, &mut metadata)?;
        let export = ExecutionArchive::decode(metadata.as_slice())?;
        assert!(matches!(export.execution.unwrap().origin.unwrap().kind, Some(v1::origin::Kind::Import(_))));
        let draft = store.read("draft")?;
        assert_eq!(draft.state, ExecutionState::Open);
        assert!(!host.capabilities("draft")?.start);
        assert!(host.start_execution("draft").await.is_err());
        drop(store);
        host.shutdown().await?;
        drop(host);

        // Restart Recovers Runs Only and Leaves the Import Draft Untouched
        let host = Host::open(root.path(), super::support::settings())?;
        assert_eq!(host.execution("draft")?, draft);
        assert_eq!(host.execution("closed-import")?, closed);
        assert_eq!(host.export_execution("imported")?, imported_bytes);
        assert!(!host.capabilities("draft")?.cancel);
        host.shutdown().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
