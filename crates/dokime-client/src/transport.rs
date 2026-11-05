/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use futures_util::{StreamExt, stream};
use harmos::MemorySize;
use harmos::domain::Portable;
use harmos::grpc::{Codec, DecodeSettings, Encode, EncodeSettings, InputEnvelope, OutputEnvelope, PreparedFrame, Transcoded, Wire};
use harmos::operations::{Direction, OperationDeclaration, Response, StreamItem};
use tonic::{
    metadata::{Ascii, MetadataValue},
    transport::Channel,
};

// Local Imports
use super::{Client, Error, ResponseStream, Result, request, stream::Input};
use dokime_operations::reports::{DownloadCycleReport, DownloadExecutionReport};

/* === Constants === */

const DECODE_RETAINED_BYTES: u64 = 67_108_864;
const ENCODE_BUFFER_BYTES: u64 = 8_192;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Client {
    pub(super) async fn download<O>(&self, message: O::Request) -> Result<ResponseStream<<O::Output as Direction>::Item>>
    where
        O: OperationDeclaration,
        O::Request: Sync,
        O::Response: Portable + MemorySize,
        <O::Output as Direction>::Item: Portable + MemorySize,
    {
        self.require(&dokime_operations::path::<O>())?;
        let report = [DownloadCycleReport::ID, DownloadExecutionReport::ID].contains(&O::ID);
        let response_bytes = if report { self.response_limit().min(2_097_152) } else { self.response_limit() };
        let codec = codec::<OutputEnvelope<<O::Output as Direction>::Item, O::Response>, Response<O::Request>>(
            response_bytes,
            self.request_limit(),
        )?;
        let mut client = tonic::client::Grpc::new(self.channel()?)
            .max_decoding_message_size(response_bytes)
            .max_encoding_message_size(self.request_limit());
        let call = self.with_timeout(O::POLICY.timeout);
        let incoming = call
            .execute(async {
                client.ready().await.map_err(|_| tonic::Status::unavailable("channel is not ready"))?;
                // A Stream Carries No Total Deadline: Opening Is Bounded Here, Each Later Frame By The Idle Deadline.
                client
                    .server_streaming(request(Response::new(message), self.inner.authorization.as_ref()), method::<O>(), codec)
                    .await
                    .map(tonic::Response::into_inner)
            })
            .await?;
        let idle = Some(O::POLICY.timeout);
        let frames = stream::unfold(Some(incoming), move |state| async move {
            let mut incoming = state?;
            let next = match idle_frame(idle, incoming.next()).await {
                Ok(next) => next,
                Err(status) => return Some((Err(status), None)),
            };
            match next {
                Some(Err(status)) => Some((Err(status), None)),
                None => Some((Err(tonic::Status::data_loss("operation stream has no completion")), None)),
                Some(Ok(frame)) => match OutputEnvelope::project(frame) {
                    Err(error) => Some((Err(tonic::Status::data_loss(error.to_string())), None)),
                    Ok(OutputEnvelope::Item(item)) => Some((value(item).map_err(status), Some(incoming))),
                    Ok(OutputEnvelope::Response(_)) => {
                        // Final Status Must Be Observed Before Reporting End Of Stream.
                        match idle_frame(idle, incoming.next()).await {
                            Err(status) => Some((Err(status), None)),
                            Ok(Some(Err(status))) => Some((Err(status), None)),
                            Ok(Some(Ok(_))) => Some((Err(tonic::Status::data_loss("operation stream continued after completion")), None)),
                            Ok(None) => None,
                        }
                    }
                },
            }
        });
        if report { ResponseStream::prefetched(self, frames, response_bytes) } else { ResponseStream::new(self, frames) }
    }

    pub(super) async fn upload<O, S>(&self, message: O::Request, items: S) -> Result<O::Response>
    where
        O: OperationDeclaration,
        O::Request: Sync,
        O::Response: Portable + MemorySize,
        <O::Input as Direction>::Item: Portable + Sync,
        S: tokio_stream::Stream<Item = <O::Input as Direction>::Item> + Send + 'static,
    {
        self.require(&dokime_operations::path::<O>())?;
        let opening = stream::once(async move { Response::new(InputEnvelope::Request(message)) });
        let frames = opening.chain(items.map(|item| Response::new(InputEnvelope::Item(item))));
        self.upload_frames::<O, _, _>(frames).await
    }

    pub(super) async fn upload_prepared<O, S>(&self, message: O::Request, items: S) -> Result<O::Response>
    where
        O: OperationDeclaration,
        S: tokio_stream::Stream<Item = PreparedFrame> + Send + 'static,
    {
        self.require(&dokime_operations::path::<O>())?;
        let opening = harmos::grpc::prepare_input_request::<O>(&message, self.request_limit()).map_err(Error::Rpc)?;
        drop(message);
        let frames = stream::once(async move { opening }).chain(items);
        self.upload_frames::<O, _, _>(frames).await
    }

    async fn upload_frames<O, E, S>(&self, frames: S) -> Result<O::Response>
    where
        O: OperationDeclaration,
        E: Encode<Transcoded> + Sync,
        S: tokio_stream::Stream<Item = E> + Send + 'static,
    {
        let call = Self { timeout: None, ..self.clone() };
        let codec = codec::<O::Response, E>(self.response_limit(), self.request_limit())?;
        let cancellation = self.inner.cancellation.child_token();
        let _cancel = cancellation.clone().drop_guard();
        let frames = Input::new(frames, cancellation);
        let progress = frames.progress();
        let mut client = tonic::client::Grpc::new(self.channel()?)
            .max_decoding_message_size(self.response_limit())
            .max_encoding_message_size(self.request_limit());
        let uploading = call.execute(async {
            client.ready().await.map_err(|_| tonic::Status::unavailable("channel is not ready"))?;
            client
                .client_streaming(request(frames, self.inner.authorization.as_ref()), method::<O>(), codec)
                .await
                .map(tonic::Response::into_inner)
        });
        let reply = self.guard(upload_idle(O::POLICY.timeout, O::ID, progress, uploading)).await??;
        value(reply)
    }
}

/* === Public Functions === */

pub(super) fn value<T>(reply: StreamItem<T>) -> Result<T> {
    // Transfer The Decoded Allocation; Never Clone A Payload To Drop Its Owner.
    reply.into_request().map(|(value, _claims)| value).map_err(|error| Error::Rpc(tonic::Status::data_loss(error.to_string())))
}

pub(super) fn codec<D, E>(response_bytes: usize, request_bytes: usize) -> Result<Codec<Transcoded, D, E>>
where
    Transcoded: Wire<D>,
    D: MemorySize + Send + 'static,
    E: Encode<Transcoded> + Sync,
{
    let response_bytes = NonZeroUsize::new(response_bytes).ok_or(Error::IncompatibleApi)?;
    let request_bytes = NonZeroUsize::new(request_bytes).ok_or(Error::IncompatibleApi)?;
    let retained_bytes =
        NonZeroUsize::new(usize::try_from(DECODE_RETAINED_BYTES).map_err(|_| Error::IncompatibleApi)?).ok_or(Error::IncompatibleApi)?;
    Codec::new(
        DecodeSettings {
            message_bytes: response_bytes,
            retained_bytes,
        },
        EncodeSettings {
            message_bytes: request_bytes,
            buffer_bytes: NonZeroUsize::new(
                request_bytes.get().min(usize::try_from(ENCODE_BUFFER_BYTES).map_err(|_| Error::IncompatibleApi)?),
            )
            .ok_or(Error::IncompatibleApi)?,
        },
    )
    .map_err(Error::Rpc)
}

pub(super) async fn unary<O>(
    channel: Channel, authorization: Option<&MetadataValue<Ascii>>, message: O::Request, response_bytes: usize, request_bytes: usize,
) -> Result<StreamItem<O::Response>>
where
    O: OperationDeclaration,
    O::Request: Send + Sync,
    O::Response: Portable + MemorySize,
{
    let path = dokime_operations::path::<O>();
    let message = Response::new(message);
    retry_unavailable(&path, || {
        let message = message.clone();
        let channel = channel.clone();
        async move {
            let codec = codec::<O::Response, Response<O::Request>>(response_bytes, request_bytes)?;
            let mut client =
                tonic::client::Grpc::new(channel).max_decoding_message_size(response_bytes).max_encoding_message_size(request_bytes);
            client.ready().await.map_err(|_| Error::Rpc(tonic::Status::unavailable("channel is not ready")))?;
            client.unary(request(message, authorization), method::<O>(), codec).await.map(tonic::Response::into_inner).map_err(Error::Rpc)
        }
    })
    .await
}

pub(super) async fn retry_unavailable<T, F: Future<Output = Result<T>>>(path: &str, mut attempt: impl FnMut() -> F) -> Result<T> {
    let result = attempt().await;
    if matches!(&result, Err(Error::Rpc(status)) if status.code() == tonic::Code::Unavailable) && read_only(path) {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        return attempt().await;
    }
    result
}

/* === Private Functions === */

async fn upload_idle<T>(
    idle: std::time::Duration, operation: &str, mut progress: tokio::sync::watch::Receiver<tokio::time::Instant>,
    uploading: impl Future<Output = Result<T>>,
) -> Result<T> {
    tokio::pin!(uploading);
    let mut observing = true;
    loop {
        let last = *progress.borrow_and_update();
        let remaining = idle.saturating_sub(tokio::time::Instant::now().saturating_duration_since(last));
        if remaining.is_zero() {
            return Err(Error::Rpc(tonic::Status::deadline_exceeded(format!("operation {operation}: upload idle deadline elapsed"))));
        }
        tokio::select! {
            biased;
            changed = progress.changed(), if observing => { observing = changed.is_ok(); }
            _ = tokio::time::sleep(remaining) => {
                if tokio::time::Instant::now().saturating_duration_since(*progress.borrow()) >= idle {
                    return Err(Error::Rpc(tonic::Status::deadline_exceeded(format!("operation {operation}: upload idle deadline elapsed"))));
                }
            }
            reply = &mut uploading => return reply,
        }
    }
}

fn method<O: OperationDeclaration>() -> tonic::codegen::http::uri::PathAndQuery {
    tonic::codegen::http::uri::PathAndQuery::try_from(dokime_operations::path::<O>()).expect("validated gRPC bindings are URI paths")
}

fn read_only(path: &str) -> bool {
    matches!(
        path,
        "/dokime.v1.Appearance/GetAppearance"
            | "/dokime.v1.Backend/GetAvailability"
            | "/dokime.v1.Backend/GetHealth"
            | "/dokime.v1.Backend/GetIdentity"
            | "/dokime.v1.Catalog/GetRecordSchema"
            | "/dokime.v1.Catalog/GetSuite"
            | "/dokime.v1.Catalog/GetTelemetrySchema"
            | "/dokime.v1.Catalog/ListRecordSchemas"
            | "/dokime.v1.Catalog/ListSuites"
            | "/dokime.v1.Catalog/ListTelemetrySchemas"
            | "/dokime.v1.Content/ListArtifacts"
            | "/dokime.v1.Content/ListExecutionFiles"
            | "/dokime.v1.Content/ReadExecutionFile"
            | "/dokime.v1.Evidence/GetCapturedRecordSchema"
            | "/dokime.v1.Evidence/ListRecords"
            | "/dokime.v1.Execution/GetCase"
            | "/dokime.v1.Execution/GetCycle"
            | "/dokime.v1.Execution/GetExecution"
            | "/dokime.v1.Execution/GetExecutionCommand"
            | "/dokime.v1.Execution/ListCaseParameters"
            | "/dokime.v1.Execution/ListCases"
            | "/dokime.v1.Execution/ListCycles"
            | "/dokime.v1.Execution/ListExecutionParameters"
            | "/dokime.v1.Execution/ListExecutionSteps"
            | "/dokime.v1.Execution/ListExecutions"
            | "/dokime.v1.Health/ListHealth"
            | "/dokime.v1.History/BrowseCycles"
            | "/dokime.v1.History/BrowseExecutions"
            | "/dokime.v1.History/GetCheckpointContext"
            | "/dokime.v1.History/GetCheckpointDocument"
            | "/dokime.v1.History/GetCheckpointEvidence"
            | "/dokime.v1.History/GetExecutionCheckpoint"
            | "/dokime.v1.History/ListCheckpointRecords"
            | "/dokime.v1.History/ListCheckpointTelemetry"
            | "/dokime.v1.History/ReadCheckpointOutput"
            | "/dokime.v1.Notes/ListNotes"
            | "/dokime.v1.Observation/GetRuntimeShell"
            | "/dokime.v1.Observation/GetTraceability"
            | "/dokime.v1.Planning/MaterializeTemplate"
            | "/dokime.v1.Planning/PreviewSelection"
            | "/dokime.v1.Plots/ListAnalyses"
            | "/dokime.v1.Plots/QuerySnapshots"
            | "/dokime.v1.Plugins/GetInstallation"
            | "/dokime.v1.Plugins/ListAdmittedPlugins"
            | "/dokime.v1.Plugins/ListInstallations"
            | "/dokime.v1.Prompts/GetDefinition"
            | "/dokime.v1.Prompts/GetPrompt"
            | "/dokime.v1.Prompts/ListDefinitions"
            | "/dokime.v1.Prompts/ListPending"
            | "/dokime.v1.Query/Aggregate"
            | "/dokime.v1.Query/AggregateSamples"
            | "/dokime.v1.Query/ListCycles"
            | "/dokime.v1.Query/ListExecutions"
            | "/dokime.v1.Query/ListRecords"
            | "/dokime.v1.Query/ListSamples"
            | "/dokime.v1.Remote/ListPairedClients"
            | "/dokime.v1.Remote/ListSessions"
            | "/dokime.v1.Remote/ReadListener"
            | "/dokime.v1.Runtime/GetHealth"
            | "/dokime.v1.Runtime/GetIdentity"
            | "/dokime.v1.Secrets/List"
            | "/dokime.v1.Secrets/Preview"
            | "/dokime.v1.Settings/GetPluginSettings"
            | "/dokime.v1.Settings/GetSettings"
            | "/dokime.v1.Settings/ListStorageSources"
            | "/dokime.v1.Settings/ReadApplicationSettings"
            | "/dokime.v1.Telemetry/ListSamples"
            | "/harmos.host.v1.Host/Catalog"
    )
}

// A Live Stream Fails Only When No Frame, Heartbeats Included, Arrives Within The Idle Deadline.
async fn idle_frame<T>(idle: Option<std::time::Duration>, next: impl Future<Output = T>) -> std::result::Result<T, tonic::Status> {
    match idle {
        Some(idle) => tokio::time::timeout(idle, next).await.map_err(|_| tonic::Status::deadline_exceeded("stream idle deadline elapsed")),
        None => Ok(next.await),
    }
}

fn status(error: Error) -> tonic::Status {
    match error {
        Error::Rpc(status) => status,
        error => tonic::Status::data_loss(error.to_string()),
    }
}

/* === Tests === */
