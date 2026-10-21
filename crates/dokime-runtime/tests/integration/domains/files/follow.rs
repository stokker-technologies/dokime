/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A live file follow races its producer's settlement: the last bytes may reach
// the viewer before the owner seals or abandons the production, or the owner may
// settle before the viewer is admitted. Every ordering reports the same terminal.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use sha2::{Digest, Sha256};

// Local Imports
use super::*;
use dokime_domain::model::content::wire;
use dokime_runtime::ExecutionFileFollower;

/* === Constants === */
// The followed native output.
const PATH: &str = "follow.bin";

// The bytes a producer writes before it settles.
const ORIGINAL: &[u8] = b"original retained prefix";

// A rewrite of the retained prefix that abandons the original production.
const REWRITTEN: &[u8] = b"changed retained prefix with extra bytes";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
// The running execution of a blocked cycle and its followed output path, holding `ORIGINAL`.
async fn producing(handle: &RuntimeHandle, cycle: &TestCycleId) -> (TestExecutionId, String) {
    let cycle = wait_for_cycle_lifecycle(handle, cycle, CycleStatus::Running).await;
    let id = cycle.test_executions[0].test_execution_id.clone();
    let target = handle.execution_file_target(&id, PATH.to_owned(), false, true).await.unwrap().unwrap();
    std::fs::write(&target.absolute_path, ORIGINAL).unwrap();
    (id, target.absolute_path)
}

async fn follow(
    handle: &RuntimeHandle, id: &TestExecutionId, identity: Option<&wire::ExecutionFileIdentity>,
) -> dokime_runtime::runtime::Result<ExecutionFileFollower> {
    Ok(handle.follow_execution_file(id, PATH, identity, 0).await?.expect("registered execution output"))
}

// Reads chunks until `length` bytes arrived; a terminal frame first fails the story.
async fn drain(follower: &mut ExecutionFileFollower, length: usize) -> Vec<u8> {
    let maximum = NonZeroUsize::new(7).unwrap();
    let mut bytes = Vec::new();
    while bytes.len() < length {
        let frame = follower.next_frame(maximum).await.unwrap().unwrap();
        let Some(wire::ExecutionFileFollowFramePayload::Chunk(chunk)) = &frame.payload else {
            panic!("follow settled before its retained bytes arrived");
        };
        assert_eq!(chunk.offset, bytes.len() as u64);
        bytes.extend_from_slice(&chunk.data);
    }
    bytes
}

async fn terminal(follower: &mut ExecutionFileFollower) -> wire::ExecutionFileFollowTerminal {
    let maximum = NonZeroUsize::new(7).unwrap();
    let frame = follower.next_frame(maximum).await.unwrap().unwrap();
    let Some(wire::ExecutionFileFollowFramePayload::Terminal(terminal)) = &frame.payload else {
        panic!("settled follow closes with its terminal frame");
    };
    let terminal = terminal.clone();
    assert!(follower.next_frame(maximum).await.unwrap().is_none());
    terminal
}

fn assert_sealed(terminal: &wire::ExecutionFileFollowTerminal, identity: &wire::ExecutionFileIdentity) {
    assert_eq!(terminal.phase, wire::ExecutionFilePhase::Finished);
    assert_eq!(terminal.length, ORIGINAL.len() as u64);
    let sealed = terminal.sealed.as_ref().expect("finished follow carries its seal");
    assert_eq!(sealed.identity.as_ref(), Some(identity));
    assert_eq!(sealed.length, ORIGINAL.len() as u64);
    let digest = Sha256::digest(ORIGINAL).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    assert_eq!(sealed.sha256, digest);
}

fn assert_abandoned(terminal: &wire::ExecutionFileFollowTerminal) {
    assert_eq!(terminal.phase, wire::ExecutionFilePhase::Abandoned);
    assert_eq!(terminal.length, ORIGINAL.len() as u64);
    assert!(terminal.sealed.is_none(), "an abandoned production is never sealed");
}

/* === Tests === */

#[tokio::test(flavor = "multi_thread")]
async fn follow_terminal_interleavings() {
    // Finished: The Last Bytes Reach The Viewer Before The Owner Seals.
    let root = tempdir().unwrap();
    let (mut owner, handle, cycle, executor) = blocked_cycle(root.path()).await;
    let (id, _) = producing(&handle, &cycle).await;
    let mut live = follow(&handle, &id, None).await.unwrap();
    assert_eq!(live.metadata().phase, wire::ExecutionFilePhase::Producing);
    let identity = live.metadata().identity.clone().unwrap();
    assert_eq!(drain(&mut live, ORIGINAL.len()).await, ORIGINAL);
    executor.release();
    wait_for_cycle_outcome(&handle, &cycle, ResultStatus::Passed).await;
    assert_sealed(&terminal(&mut live).await, &identity);

    // Finished: The Owner Seals Before The Viewer Is Admitted.
    let mut settled = follow(&handle, &id, Some(&identity)).await.unwrap();
    assert_eq!(settled.metadata().phase, wire::ExecutionFilePhase::Finished);
    assert_eq!(drain(&mut settled, ORIGINAL.len()).await, ORIGINAL);
    assert_sealed(&terminal(&mut settled).await, &identity);
    drop((live, settled, handle));
    owner.shutdown().await.unwrap();

    // Abandoned: The Last Bytes Reach The Viewer Before The Source Fails.
    let root = tempdir().unwrap();
    let (mut owner, handle, cycle, executor) = blocked_cycle(root.path()).await;
    let (id, source) = producing(&handle, &cycle).await;
    let mut live = follow(&handle, &id, None).await.unwrap();
    let identity = live.metadata().identity.clone().unwrap();
    assert_eq!(drain(&mut live, ORIGINAL.len()).await, ORIGINAL);
    std::fs::write(&source, REWRITTEN).unwrap();
    executor.release();

    // Abandoned: The Owner Retains The Interrupted Prefix Before The Viewer Is Admitted.
    // Admission refuses the failed source until capture settles, then never reports it finished.
    // Admission Is Retried On Each Committed State Change Until Capture Settles.
    let mut settled = settle(&handle, || async {
        match follow(&handle, &id, Some(&identity)).await {
            Ok(settled) => Some(settled),
            Err(dokime_runtime::runtime::Error::Operation(_)) => None,
            Err(error) => panic!("failed source refuses admission, got {error:?}"),
        }
    })
    .await;
    assert_eq!(settled.metadata().identity.as_ref(), Some(&identity));
    assert_eq!(settled.metadata().phase, wire::ExecutionFilePhase::Abandoned);
    assert_eq!(drain(&mut settled, ORIGINAL.len()).await, ORIGINAL);
    assert_abandoned(&terminal(&mut settled).await);
    assert_abandoned(&terminal(&mut live).await);
    drop((live, settled, handle));
    // The capture resident retains the source failure through shutdown.
    assert!(owner.shutdown().await.is_err());
}
