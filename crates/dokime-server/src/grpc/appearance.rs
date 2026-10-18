/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod assets;
mod convert;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use harmos::operations::{Context, Operation, StreamOutput};
use prost::Message;
use tonic::Status;

// Local Imports
use crate::grpc::transcode::Transcode;
use dokime_domain::operations::appearance as wire;
use dokime_domain::operations::content;
use dokime_runtime::RuntimeHandle;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    pub runtime: RuntimeHandle,
    pub max_message_bytes: usize,
}

struct Download {
    resources: Arc<Resources>,
}

struct Read {
    resources: Arc<Resources>,
}

/* === Enums === */

#[harmos::catalog]
enum Operations {
    Read(Read),
    Download(Download),
}

/* === Traits === */

/* === Implementations === */

impl Operation for Read {
    type Declaration = wire::GetAppearance;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, _: &mut wire::GetAppearance, _: &mut (),
    ) -> Result<harmos::operations::Response<dokime_domain::model::appearance::wire::AppearanceSnapshot>, Status> {
        let captured = self
            .resources
            .runtime
            .capture_appearance(source_budget(self.resources.max_message_bytes))
            .map_err(|_| failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::TooLarge))?;
        let mut logos = Vec::with_capacity(captured.frame.logos.len());
        for (logo, source) in captured.frame.logos.iter().zip(captured.logos) {
            let source = source.ok_or_else(|| failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::AssetMissing))?;
            let asset = assets::capture(source, logo.plugin_id.to_string(), logo.asset_id.to_string()).await?;
            logos.push(dokime_domain::model::appearance::wire::AppearanceLogo {
                variant: logo.variant,
                asset: Some(asset.metadata),
                alt: logo.alt.clone(),
            });
        }
        let frame = convert::appearance(captured.frame, logos)
            .map_err(|_| failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::TooLarge))?;
        if frame.wire_len()? > self.resources.max_message_bytes {
            return Err(failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::TooLarge));
        }
        Ok((frame).into())
    }
}

impl Operation for Download {
    type Declaration = content::DownloadAppearanceAsset;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<dokime_domain::model::content::wire::AppearanceAssetFrame>;
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut content::DownloadAppearanceAsset, output: &mut Self::Endpoints,
    ) -> Result<harmos::operations::Response<()>, Status> {
        assets::download(&self.resources.runtime, std::mem::take(request), self.resources.max_message_bytes, output).await?;
        Ok(().into())
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, max_message_bytes: usize) -> Result<(), super::Error> {
    runtime.operations().register_wire_operations::<Operations, Resources>(
        "dokime",
        dokime_runtime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), max_message_bytes }),
    )?;
    Ok(())
}

pub(super) fn historical(
    frame: dokime_runtime::domains::installed_plugins::RuntimeAppearanceFrame,
) -> Result<dokime_domain::model::appearance::wire::AppearanceSnapshot, Status> {
    convert::historical_memory(&frame).map_err(|_| failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::TooLarge))?;
    convert::appearance(frame, Vec::new()).map_err(|_| failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::TooLarge))
}

/* === Private Functions === */

fn source_budget(max_message_bytes: usize) -> usize {
    max_message_bytes.min(4 * 1024 * 1024)
}

fn failure(kind: dokime_domain::model::appearance::wire::AppearanceFailureKind) -> Status {
    use dokime_domain::model::appearance::wire::AppearanceFailureKind::*;
    let (code, message) = match kind {
        AssetMissing => (tonic::Code::NotFound, "appearance asset is unavailable; refresh appearance"),
        AssetChanged => (tonic::Code::FailedPrecondition, "appearance asset changed; refresh appearance"),
        TooLarge => (tonic::Code::ResourceExhausted, "appearance exceeds its size budget"),
        InvalidReference => (tonic::Code::InvalidArgument, "invalid appearance asset reference"),
        _ => (tonic::Code::Unavailable, "appearance asset cannot be read"),
    };
    let envelope = tonic_types::Status {
        code: code as i32,
        message: message.into(),
        // A Detail The Transcoder Refuses Is Omitted; The Status Itself Still Reaches The Caller.
        details: crate::grpc::transcode::Transcode::wire_bytes(&dokime_domain::model::appearance::wire::AppearanceFailure { kind })
            .ok()
            .map(|value| prost_types::Any { type_url: "type.googleapis.com/dokime.v1.AppearanceFailure".into(), value })
            .into_iter()
            .collect(),
    };
    Status::with_details(code, message, envelope.encode_to_vec().into())
}

/* === Tests === */
