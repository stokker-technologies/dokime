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

use std::future::Future;
use crate::client::{Context, Output};
use tokio_stream::StreamExt;

// Standard Library Imports

// Third Party Imports

// Local Imports


/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

pub enum Input<Q> { Request(Q), Cancel, Finish }

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn unary<I, Q, R, O, F>(context: Context, mut input: tonic::Streaming<I>, decode: impl Fn(I) -> Option<Input<Q>> + Send + 'static, encode: impl Fn(R) -> O + Send + 'static, handle: impl Fn(Context, Q) -> F + Send + 'static) -> Result<Output<O>, tonic::Status>
where I: Send + 'static, Q: Send + 'static, R: Send + 'static, O: Send + 'static, F: Future<Output = crate::Result<R>> + Send + 'static {
    let first = tokio::time::timeout(context.settings.read_timeout, input.message()).await.map_err(|_| tonic::Status::deadline_exceeded("Send the Request Before the Read Deadline"))??;
    let Some(Input::Request(request)) = first.and_then(&decode) else { return Err(tonic::Status::invalid_argument("Send Exactly One Request First")); };
    let (sender, receiver) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
    tokio::spawn(async move {
        let work = handle(context, request);
        tokio::pin!(work);
        let mut finished = false;
        loop {
            tokio::select! {
                biased;
                _ = sender.closed() => break,
                message = input.message(), if !finished => match message {
                    Ok(Some(message)) => match decode(message) {
                        Some(Input::Finish) => finished = true,
                        Some(Input::Cancel) => { let _ = sender.send(Err(tonic::Status::cancelled("Call Cancelled; Inspect Any Committed Mutation"))).await; break; },
                        _ => { let _ = sender.send(Err(tonic::Status::invalid_argument("Send Finish or Cancel After the Request"))).await; break; }
                    },
                    Ok(None) => finished = true,
                    Err(error) => { let _ = sender.send(Err(error)).await; break; }
                },
                result = &mut work => { let _ = sender.send(result.map(&encode).map_err(super::status)).await; break; }
            }
        }
    });
    Ok(tokio_stream::wrappers::ReceiverStream::new(receiver))
}

pub async fn map_stream<A: Send + 'static, B: Send + 'static>(context: &Context, mut input: Output<A>, map: impl Fn(A) -> B + Send + 'static) -> Output<B> {
    let (sender, receiver) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
    tokio::spawn(async move { while let Some(item) = input.next().await { if sender.send(item.map(&map)).await.is_err() { break; } } });
    tokio_stream::wrappers::ReceiverStream::new(receiver)
}

/* === Private Functions === */

/* === Tests === */
