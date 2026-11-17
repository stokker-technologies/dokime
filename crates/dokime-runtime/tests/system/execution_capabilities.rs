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
use std::error::Error;
use std::io::Cursor;
use std::io::Read;

// Third Party Imports
use prost::Message;

// Local Imports
use dokime_runtime::Host;
use dokime_storage::ExecutionArchive;

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
fn execution_capabilities() -> Result<(), Box<dyn Error>> {
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
            let mut archive = zip::ZipArchive::new(Cursor::new(host.export_execution(&execution).await?))?;
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

            host.shutdown().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}
