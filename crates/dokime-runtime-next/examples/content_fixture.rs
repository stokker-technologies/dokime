/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Features Shared by Host Stories

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::{prelude::*,v1};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main()->Result<()> {
    let template=Template::new("content.capture","Capture Content").step(Step::new("Capture",capture));
    let plugin=Plugin::new("content_fixture","1.0.0").suite(Suite::new("content","Content").template(template)).requires_host::<v1::PublishArtifact>();
    let plugin=if std::env::args().any(|argument|argument=="--remote"){plugin.remote()}else{plugin};
    plugin.serve()
}
async fn capture(ctx:&Context)->Result<()> {
    // Start an Ordered Transfer
    let request=v1::PublishArtifact {transfer_id:"capture".into(),name:"producer.txt".into(),mime:"text/plain".into(),data:b"old prefix".to_vec(),..Default::default()};
    ctx.host(request.clone()).await?;
    ctx.host(request).await?;

    // Refuse a Sequence Gap
    let request=v1::PublishArtifact {transfer_id:"capture".into(),name:"producer.txt".into(),mime:"text/plain".into(),sequence:3,data:b"gap".to_vec(),..Default::default()};
    if ctx.host(request).await.is_ok(){return Err(Error::Invalid("Capture: Sequence Gap Was Accepted".into()));}

    // Replace the Prefix Within the Same Transfer
    let request=v1::PublishArtifact {transfer_id:"capture".into(),name:"producer.txt".into(),mime:"text/plain".into(),sequence:1,data:b"replacement".to_vec(),replace:true,..Default::default()};
    ctx.host(request).await?;

    // Commit an Explicit Terminal Chunk
    let request=v1::PublishArtifact {transfer_id:"capture".into(),name:"producer.txt".into(),mime:"text/plain".into(),sequence:2,data:b"\n".to_vec(),last:true,..Default::default()};
    let receipt=ctx.host(request).await?;
    if !receipt.sealed || receipt.size_bytes!=12 {return Err(Error::Invalid("Capture: Wrong Terminal Receipt".into()));}

    // Abort Another Transfer Without Publishing Its Prefix
    let request=v1::PublishArtifact {transfer_id:"aborted".into(),name:"aborted.txt".into(),mime:"text/plain".into(),data:b"unpublished".to_vec(),..Default::default()};
    ctx.host(request).await?;
    let request=v1::PublishArtifact {transfer_id:"aborted".into(),name:"aborted.txt".into(),mime:"text/plain".into(),sequence:1,last:true,aborted:true,..Default::default()};
    let receipt=ctx.host(request).await?;
    if receipt.sealed {return Err(Error::Invalid("Capture: Aborted Transfer Was Published".into()));}
    Ok(())
}

/* === Tests === */
