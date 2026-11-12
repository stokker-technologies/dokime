/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Conversation

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::future::Future;

// Third Party Imports
use tokio_stream::StreamExt;

// Local Imports
use crate::client::{Context, Output};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

pub enum Input<Q> {
    Request(Q),
    Cancel,
    Finish,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn unary<I, Q, R, O, F>(
    context: Context, mut input: tonic::Streaming<I>, decode: impl Fn(I) -> Option<Input<Q>> + Send + 'static,
    encode: impl Fn(R) -> O + Send + 'static, handle: impl Fn(Context, Q) -> F + Send + 'static,
) -> Result<Output<O>, tonic::Status>
where
    I: Send + 'static,
    Q: Send + 'static,
    R: prost::Message + Send + 'static,
    O: Send + 'static,
    F: Future<Output = crate::Result<R>> + Send + 'static,
{
    let first = tokio::time::timeout(context.settings.read_timeout, input.message())
        .await
        .map_err(|_| tonic::Status::deadline_exceeded("Send the Request Before the Read Deadline"))??;
    let Some(Input::Request(request)) = first.and_then(&decode) else {
        return Err(tonic::Status::invalid_argument("Send Exactly One Request First"));
    };
    let (sender, receiver) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
    tokio::spawn(async move {
        let reply_context = context.clone();
        let work = handle(context, request);
        tokio::pin!(work);
        let mut finished = false;
        let mut input_closed = false;
        loop {
            tokio::select! {
                biased;
                _ = sender.closed() => break,
                message = input.message(), if !input_closed => match message {
                    Ok(Some(message)) => match decode(message) {
                        Some(Input::Finish) if !finished => finished = true,
                        Some(Input::Cancel) if !finished => { let _ = sender.send(Err(tonic::Status::cancelled("Call Cancelled; Inspect Any Committed Mutation"))).await; break; },
                        _ => { let _ = sender.send(Err(tonic::Status::invalid_argument("Send Finish or Cancel After the Request"))).await; break; }
                    },
                    Ok(None) => input_closed = true,
                    Err(error) => { let _ = sender.send(Err(error)).await; break; }
                },
                result = &mut work => { let _ = sender.send(result.and_then(|reply| super::bounded_reply(&reply_context,reply)).map(&encode).map_err(super::status)).await; break; }
            }
        }
    });
    Ok(tokio_stream::wrappers::ReceiverStream::new(receiver))
}

pub async fn map_stream<A: Send + 'static, B: Send + 'static>(
    context: &Context, mut input: Output<A>, map: impl Fn(A) -> B + Send + 'static,
) -> Output<B> {
    let (sender, receiver) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
    tokio::spawn(async move {
        while let Some(item) = input.next().await {
            if sender.send(item.map(&map)).await.is_err() {
                break;
            }
        }
    });
    tokio_stream::wrappers::ReceiverStream::new(receiver)
}

pub async fn streaming<I, Q, A, O, F>(
    context: Context, mut input: tonic::Streaming<I>, decode: impl Fn(I) -> Option<Input<Q>> + Send + 'static,
    encode: impl Fn(A) -> O + Send + 'static, handle: impl Fn(Context, Q) -> F + Send + 'static,
) -> Result<Output<O>, tonic::Status>
where
    I: Send + 'static,
    Q: Send + 'static,
    A: Send + 'static,
    O: Send + 'static,
    F: Future<Output = crate::Result<Output<A>>> + Send + 'static,
{
    let first = tokio::time::timeout(context.settings.read_timeout, input.message())
        .await
        .map_err(|_| tonic::Status::deadline_exceeded("Send the Request Before the Read Deadline"))??;
    let Some(Input::Request(request)) = first.and_then(&decode) else {
        return Err(tonic::Status::invalid_argument("Send Exactly One Request First"));
    };
    let capacity = context.settings.stream_buffer;
    let mut output = handle(context, request).await.map_err(super::status)?;
    let (sender, receiver) = tokio::sync::mpsc::channel(capacity);
    tokio::spawn(async move {
        let mut finished = false;
        let mut input_closed = false;
        loop {
            tokio::select! {
                biased;
                _ = sender.closed() => break,
                message = input.message(), if !input_closed => match message {
                    Ok(Some(message)) => match decode(message) {
                        Some(Input::Finish) if !finished => finished = true,
                        Some(Input::Cancel) if !finished => { let _ = sender.send(Err(tonic::Status::cancelled("Watch Cancelled; Execution Continues"))).await; break; },
                        _ => { let _ = sender.send(Err(tonic::Status::invalid_argument("Send Finish or Cancel After the Request"))).await; break; },
                    },
                    Ok(None) => input_closed = true,
                    Err(error) => { let _ = sender.send(Err(error)).await; break; },
                },
                frame = output.next() => match frame { Some(frame) => if sender.send(frame.map(&encode)).await.is_err() { break; }, None => break },
            }
        }
    });
    Ok(tokio_stream::wrappers::ReceiverStream::new(receiver))
}

/* === Private Functions === */

/* === Tests === */
