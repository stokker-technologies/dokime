/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Invocation Envelopes

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{
    Error, Result,
    client::{Context, Output},
};
use tokio_stream::StreamExt;

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

pub async fn unary<Q, I, R, O, F, W>(
    context: &Context, mut input: tonic::Streaming<I>, decode: fn(I) -> Option<Input<Q>>, work: F, response: fn(R) -> O,
) -> Result<Output<O>>
where
    Q: Send + 'static,
    I: Send + 'static,
    O: Send + 'static,
    R: Send + 'static,
    F: FnOnce(Context, Q) -> W + Send + 'static,
    W: std::future::Future<Output = Result<R>> + Send,
{
    let first = super::complete("Run Operation Request", context.settings.read_timeout, async {
        input.message().await.map_err(|error| Error::Invalid(error.to_string()))
    })
    .await?;
    let Some(Input::Request(request)) = first.and_then(decode) else {
        return Err(Error::Invalid("Run Operation: Send Exactly One Request First".into()));
    };
    let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
    let context = context.clone();
    tokio::spawn(async move {
        let future = work(context, request);
        tokio::pin!(future);
        tokio::select! {
            biased;
            result = &mut future => {
                let result = result.map(response).map_err(super::status);
                let _ = sender.send(result).await;

            }
            _ = sender.closed() => (),
            next = input.message() => match next {
                Ok(Some(value)) => match decode(value) {
                    Some(Input::Cancel) => { let _ = sender.send(Err(tonic::Status::cancelled("Run Operation: Cancelled by the Client"))).await;  }
                    Some(Input::Finish) => { let result = future.await.map(response).map_err(super::status); let _ = sender.send(result).await;  }
                    _ => { let _ = sender.send(Err(tonic::Status::invalid_argument("Run Operation: Send Only Finish or Cancel After the Request"))).await;  }
                },
                Ok(None) => { let result = future.await.map(response).map_err(super::status); let _ = sender.send(result).await;  }
                Err(error) => { let _ = sender.send(Err(error)).await;  }
            }
        }
    });
    Ok(tokio_stream::wrappers::ReceiverStream::new(output))
}

pub async fn streaming<Q, I, R, O, F, W>(
    context: &Context, mut input: tonic::Streaming<I>, decode: fn(I) -> Option<Input<Q>>, work: F, frame: fn(R) -> O, terminal: fn() -> O,
) -> Result<Output<O>>
where
    Q: Send + 'static,
    I: Send + 'static,
    R: Send + 'static,
    O: Send + 'static,
    F: FnOnce(Context, Q) -> W + Send + 'static,
    W: std::future::Future<Output = Result<Output<R>>> + Send,
{
    let first = super::complete("Run Operation Request", context.settings.read_timeout, async {
        input.message().await.map_err(|error| Error::Invalid(error.to_string()))
    })
    .await?;
    let Some(Input::Request(request)) = first.and_then(decode) else {
        return Err(Error::Invalid("Run Operation: Send Exactly One Request First".into()));
    };
    let stream = work(context.clone(), request).await?;
    let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
    tokio::spawn(async move {
        let mut stream = stream;
        let mut input_open = true;
        loop {
            tokio::select! {
                _ = sender.closed() => break,
                next = stream.next() => match next {
                    Some(value) => { let failed = value.is_err(); if sender.send(value.map(frame)).await.is_err() || failed { break; } }
                    None => break,
                },
                next = input.message(), if input_open => match next {
                    Ok(Some(value)) => match decode(value) {
                        Some(Input::Cancel | Input::Finish) => { let _ = sender.send(Ok(terminal())).await; break; }
                        _ => { let _ = sender.send(Err(tonic::Status::invalid_argument("Run Operation: Send Only Finish or Cancel After the Request"))).await; break; }
                    },
                    Ok(None) => input_open = false,
                    Err(error) => { let _ = sender.send(Err(error)).await; break; }
                }
            }
        }
    });
    Ok(tokio_stream::wrappers::ReceiverStream::new(output))
}

pub async fn finite<Q, I, R, O, F, W>(
    context: &Context, mut input: tonic::Streaming<I>, decode: fn(I) -> Option<Input<Q>>, work: F, frame: fn(R) -> O, _terminal: fn() -> O,
) -> Result<Output<O>>
where
    Q: Send + 'static,
    I: Send + 'static,
    R: Send + 'static,
    O: Send + 'static,
    F: FnOnce(Context, Q) -> W + Send + 'static,
    W: std::future::Future<Output = Result<Output<R>>> + Send,
{
    let first = super::complete("Run Operation Request", context.settings.read_timeout, async {
        input.message().await.map_err(|error| Error::Invalid(error.to_string()))
    })
    .await?;
    let Some(Input::Request(request)) = first.and_then(decode) else {
        return Err(Error::Invalid("Run Operation: Send Exactly One Request First".into()));
    };
    let stream = work(context.clone(), request).await?;
    let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
    tokio::spawn(async move {
        let mut stream = stream;
        let mut input_open = true;
        loop {
            tokio::select! {
                _ = sender.closed() => break,
                next = stream.next() => match next {
                    Some(value) => { let failed = value.is_err(); if sender.send(value.map(frame)).await.is_err() || failed { break; } }
                    None => break,
                },
                next = input.message(), if input_open => match next {
                    Ok(Some(value)) => match decode(value) {
                        Some(Input::Cancel) => { let _ = sender.send(Err(tonic::Status::cancelled("Run Download: Cancelled by the Client"))).await; break; }
                        Some(Input::Finish) => input_open = false,
                        _ => { let _ = sender.send(Err(tonic::Status::invalid_argument("Run Operation: Send Only Finish or Cancel After the Request"))).await; break; }
                    },
                    Ok(None) => input_open = false,
                    Err(error) => { let _ = sender.send(Err(error)).await; break; }
                }
            }
        }
    });
    Ok(tokio_stream::wrappers::ReceiverStream::new(output))
}

/* === Private Functions === */

/* === Tests === */
