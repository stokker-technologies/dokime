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

async fn terminal(follower: &mut ExecutionFileFollower) -> harmos::operations::Response<wire::ExecutionFileFollowFrame> {
    let maximum = NonZeroUsize::new(7).unwrap();
    let frame = follower.next_frame(maximum).await.unwrap().unwrap();
    let Some(wire::ExecutionFileFollowFramePayload::Terminal(_)) = &frame.payload else {
        panic!("settled follow closes with its terminal frame");
    };
    assert!(follower.next_frame(maximum).await.unwrap().is_none());
    frame
}

fn assert_sealed(frame: &harmos::operations::Response<wire::ExecutionFileFollowFrame>, identity: &wire::ExecutionFileIdentity) {
    let Some(wire::ExecutionFileFollowFramePayload::Terminal(terminal)) = &frame.payload else {
        panic!();
    };
    assert_eq!(terminal.phase, wire::ExecutionFilePhase::Finished);
    assert_eq!(terminal.length, ORIGINAL.len() as u64);
    let sealed = terminal.sealed.as_ref().expect("finished follow carries its seal");
    assert_eq!(sealed.identity.as_ref(), Some(identity));
    assert_eq!(sealed.length, ORIGINAL.len() as u64);
    let digest = Sha256::digest(ORIGINAL).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    assert_eq!(sealed.sha256, digest);
}

fn assert_abandoned(frame: &harmos::operations::Response<wire::ExecutionFileFollowFrame>) {
    let Some(wire::ExecutionFileFollowFramePayload::Terminal(terminal)) = &frame.payload else {
        panic!();
    };
    assert_eq!(terminal.phase, wire::ExecutionFilePhase::Abandoned);
    assert_eq!(terminal.length, ORIGINAL.len() as u64);
    assert!(terminal.sealed.is_none(), "an abandoned production is never sealed");
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn live_growth_resumes_one_identity_and_seals_exact_prefix() {
    use std::io::Write as _;
    let root = tempdir().unwrap();
    let (mut owner, handle, cycle, executor) = blocked_cycle(root.path()).await;
    let (id, path) = producing(&handle, &cycle).await;
    let mut initial = follow(&handle, &id, None).await.unwrap();
    let identity = initial.metadata().identity.clone().unwrap();
    assert_eq!(drain(&mut initial, ORIGINAL.len()).await, ORIGINAL);

    let suffix = b" appended bytes";
    let mut writer = std::fs::OpenOptions::new().append(true).open(path).unwrap();
    writer.write_all(suffix).unwrap();
    drop(writer);
    let mut resumed = handle.follow_execution_file(&id, PATH, Some(&identity), ORIGINAL.len() as u64).await.unwrap().unwrap();
    assert_eq!(resumed.metadata().identity.as_ref(), Some(&identity));
    let mut delivered = Vec::new();
    while delivered.len() < suffix.len() {
        let frame = resumed.next_frame(NonZeroUsize::new(5).unwrap()).await.unwrap().unwrap();
        let Some(wire::ExecutionFileFollowFramePayload::Chunk(chunk)) = &frame.payload else {
            panic!("growing source settled before its suffix");
        };
        assert_eq!(chunk.offset, (ORIGINAL.len() + delivered.len()) as u64);
        delivered.extend_from_slice(&chunk.data);
    }
    assert_eq!(delivered, suffix);

    executor.release();
    wait_for_cycle_outcome(&handle, &cycle, ResultStatus::Passed).await;
    let frame = terminal(&mut resumed).await;
    let Some(wire::ExecutionFileFollowFramePayload::Terminal(value)) = &frame.payload else { panic!() };
    assert_eq!(value.phase, wire::ExecutionFilePhase::Finished);
    assert_eq!(value.length, (ORIGINAL.len() + suffix.len()) as u64);
    let sealed = value.sealed.as_ref().unwrap();
    assert_eq!(sealed.identity.as_ref(), Some(&identity));
    let bytes = [ORIGINAL, suffix.as_slice()].concat();
    let expected = Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    assert_eq!(sealed.sha256, expected);
    drop((initial, resumed, handle));
    owner.shutdown().await.unwrap();
}

#[dokime_tracing::trace_test]
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
