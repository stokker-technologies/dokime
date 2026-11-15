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

// Third Party Imports

// Local Imports
use dokime::Import;
use dokime::Origin;
use dokime::Termination;
use dokime::client::DeclaredOutcome;
use dokime::client::ExecutionOutcome;
use dokime::client::ResultStatus;
use dokime::client::UnevaluatedOutcome;
use dokime_protocol::host::execution_outcome::Attribution as ExecutionOutcomeAttribution;
use dokime_protocol::model::origin::Kind as OriginKind;
use dokime_runtime::Host;
use dokime_storage::Execution;
use dokime_storage::ExecutionArchive;
use dokime_storage::ExecutionState;
use dokime_storage::Store;
use prost::Message;
use std::error::Error as StdError;
use std::io::Cursor;
use std::io::Read;

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

#[test]
fn execution_capabilities() -> Result<(), Box<dyn StdError>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Inspect and Cancel an Active Plugin Run
            let root = super::support::tempdir()?;
            let host = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
            let entry = host.add_managed("execution", &super::support::execution_fixture()?, &[]).await?;
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
            let mut archive = zip::ZipArchive::new(Cursor::new(host.export_execution(&execution)?))?;
            let mut metadata = Vec::new();
            Read::read_to_end(&mut archive.by_name("evidence.pb")?, &mut metadata)?;
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
            let import = Origin {
                kind: Some(OriginKind::Import(Import {
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
                Termination::Completed,
                ExecutionOutcome {
                    attribution: Some(ExecutionOutcomeAttribution::Declared(DeclaredOutcome {
                        status: ResultStatus::Passed as i32,
                        importer: "sample.importer".into(),
                        ..Default::default()
                    })),
                },
            )?;
            store.seal("imported")?;
            store.close(
                "closed-import",
                Termination::Completed,
                ExecutionOutcome {
                    attribution: Some(ExecutionOutcomeAttribution::NotEvaluated(UnevaluatedOutcome {
                        termination: Termination::Completed as i32,
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
            let mut archive = zip::ZipArchive::new(Cursor::new(&imported_bytes))?;
            let mut metadata = Vec::new();
            Read::read_to_end(&mut archive.by_name("evidence.pb")?, &mut metadata)?;
            let export = ExecutionArchive::decode(metadata.as_slice())?;
            assert!(matches!(export.execution.unwrap().origin.unwrap().kind, Some(OriginKind::Import(_))));
            let draft = store.read("draft")?;
            assert_eq!(draft.state, ExecutionState::Open);
            assert!(!host.capabilities("draft")?.start);
            assert!(host.start_execution("draft").await.is_err());
            drop(store);
            host.shutdown().await?;
            drop(host);

            // Restart Recovers Runs Only and Leaves the Import Draft Untouched
            let host = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
            assert_eq!(host.execution("draft")?, draft);
            assert_eq!(host.execution("closed-import")?, closed);
            assert_eq!(host.export_execution("imported")?, imported_bytes);
            assert!(!host.capabilities("draft")?.cancel);
            host.shutdown().await?;
            Ok::<(), Box<dyn StdError>>(())
        })
        .await?
    }))
}
