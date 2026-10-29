/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verify cancellation retains bounded read and mutation ownership independently.

#[path = "../../../src/grpc/plugins/admission.rs"]
mod admission;

pub(crate) async fn cancelled_admission_ownership() {
    let admission = admission::Admission::new();
    let read = admission.read().unwrap();
    let (read_started, started) = tokio::sync::oneshot::channel();
    let (release_read, released) = tokio::sync::oneshot::channel();
    let (read_finished, finished) = tokio::sync::oneshot::channel();
    let reader = tokio::task::spawn_blocking(move || {
        let permit = read;
        read_started.send(()).unwrap();
        released.blocking_recv().unwrap();
        drop(permit);
        read_finished.send(()).unwrap();
    });
    started.await.unwrap();
    reader.abort();
    assert_eq!(admission.read().unwrap_err().code(), tonic::Code::ResourceExhausted);
    let mutation = admission.mutation().unwrap();
    let (mutation_started, started) = tokio::sync::oneshot::channel();
    let (release_mutation, released) = tokio::sync::oneshot::channel();
    let (mutation_finished, completed) = tokio::sync::oneshot::channel();
    let writer = tokio::task::spawn_blocking(move || {
        let permit = mutation;
        mutation_started.send(()).unwrap();
        released.blocking_recv().unwrap();
        drop(permit);
        mutation_finished.send(()).unwrap();
    });
    started.await.unwrap();
    writer.abort();
    assert_eq!(admission.mutation().unwrap_err().code(), tonic::Code::ResourceExhausted);
    release_read.send(()).unwrap();
    finished.await.unwrap();
    let next_read = admission.read().unwrap();
    assert_eq!(admission.mutation().unwrap_err().code(), tonic::Code::ResourceExhausted);
    release_mutation.send(()).unwrap();
    completed.await.unwrap();
    let next_mutation = admission.mutation().unwrap();
    drop((next_read, next_mutation));
    reader.await.unwrap();
    writer.await.unwrap();
}
