/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Follow a Registered File Production

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::{Context, Output}};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.content.follow_execution_file";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ContentFollowExecutionFileRequest,v1::ContentFollowExecutionFileOutput>(ID,settings.watch_timeout)
}
pub async fn handle(context: &Context, request: v1::ContentFollowExecutionFileRequest) -> Result<Output<v1::ContentFollowExecutionFileOutput>> {
    let maximum=super::content_transfer::maximum(context,request.max_bytes)?;
    let store=context.host.store();
    let mut changed=store.subscribe();
    let entry=if let Some(identity)=&request.identity { store.content(&identity.file_id)? } else { store.execution_file(&request.execution_id,&request.relative_path)? };
    let identity=super::content_transfer::file_identity(context,&entry);
    if entry.execution!=request.execution_id || entry.path!=request.relative_path || request.identity.as_ref().is_some_and(|given|given!=&identity)
        || request.offset!=0 && request.identity.is_none() || request.offset>entry.length {
        return Err(Error::Invalid("Follow Execution File: Restart at Offset Zero or Resume with Its Original Identity and Committed Offset".into()));
    }
    let metadata=entry.file_metadata()?;
    let frame=v1::ExecutionFileFollowMetadata {identity:Some(identity.clone()),content_type:metadata.content_type.unwrap_or_default(),kind:metadata.kind,phase:entry.phase as i32,available_length:entry.length,offset:request.offset};
    let (sender,output)=tokio::sync::mpsc::channel(context.settings.stream_buffer);
    let timeout=context.settings.watch_timeout;
    tokio::spawn(async move {
        let work=async {
            sender.send(Ok(item(v1::execution_file_follow_frame_payload::Kind::Metadata(frame)))).await.map_err(|_|Error::Cancelled)?;
            let mut offset=request.offset;
            loop {
                let entry=store.content(&identity.file_id)?;
                while offset<entry.length {
                    let data=store.content_window(&identity.file_id,offset,maximum.min((entry.length-offset) as usize))?;
                    if data.is_empty() {return Err(Error::Invalid("Follow Execution File: Restore the Original Retained Production".into()));}
                    let chunk=v1::ArtifactChunk {offset,data};
                    offset+=chunk.data.len() as u64;
                    sender.send(Ok(item(v1::execution_file_follow_frame_payload::Kind::Chunk(chunk)))).await.map_err(|_|Error::Cancelled)?;
                }
                let closed=store.metadata(&entry.execution)?.state!=dokime_storage::ExecutionState::Open;
                if entry.phase!=v1::ExecutionFilePhase::Producing || closed {
                    let phase=if entry.phase==v1::ExecutionFilePhase::Producing {v1::ExecutionFilePhase::Abandoned} else {entry.phase};
                    let sealed=if entry.sha256.is_empty() {None} else {Some(v1::ExecutionFileSeal {identity:Some(identity.clone()),length:entry.length,sha256:entry.sha256,position:None})};
                    let terminal=v1::ExecutionFileFollowTerminal {phase:phase as i32,length:entry.length,sealed};
                    sender.send(Ok(item(v1::execution_file_follow_frame_payload::Kind::Terminal(terminal)))).await.map_err(|_|Error::Cancelled)?;
                    sender.send(Ok(v1::ContentFollowExecutionFileOutput {value:Some(v1::content_follow_execution_file_output::Value::Response(v1::ClientEmpty {}))})).await.map_err(|_|Error::Cancelled)?;
                    return Ok(());
                }
                tokio::select! {
                    _=sender.closed()=>return Ok(()),
                    result=changed.changed()=>if result.is_err(){return Err(Error::Invalid("Follow Execution File: Reconnect to the Host".into()));}
                }
            }
        };
        if let Err(error)=super::complete(ID,timeout,work).await {let _=sender.send(Err(super::status(error))).await;}
    });
    Ok(tokio_stream::wrappers::ReceiverStream::new(output))
}

/* === Private Functions === */

fn item(kind: v1::execution_file_follow_frame_payload::Kind) -> v1::ContentFollowExecutionFileOutput {
    v1::ContentFollowExecutionFileOutput {value:Some(v1::content_follow_execution_file_output::Value::Item(v1::ExecutionFileFollowFrame {payload:Some(v1::ExecutionFileFollowFramePayload {kind:Some(kind)})}))}
}

/* === Tests === */
