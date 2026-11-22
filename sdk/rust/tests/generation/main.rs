/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Compile the Generated Client and Plugin Barrels as a Consumer.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::FileSource;
use dokime::client::Client;
use dokime::templates::Context;
use std::error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// Compiler Story: No Host, Runtime, Storage or Fixtures Are Required.
pub async fn generated_surface(client: &Client, context: &Context) -> Result<(), Box<dyn error::Error>> {
    let _ = client.plugins().list().await?;
    let _ = context.capture(FileSource::default(), String::new()).await?;
    let _ = context.artifact("capture.bin", Vec::new(), "application/octet-stream").await?;
    let _ = context.statistics(String::new(), String::new(), String::new()).await?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */

#[test]
fn heat_strip_document_builder_roundtrips_through_the_public_sdk() {
    use prost::Message;
    let samples = dokime::FieldReference::new("samples");
    let plot = dokime::builders::Plot::heat_strip(samples.field("value"))
        .cells(samples.field("cell"))
        .facet(samples.field("facet"))
        .reduce(dokime::Reduction::Mean);
    let document = dokime::Document::new("Heat Strip").section(dokime::model::Section::new("Values").plot(plot));
    assert_eq!(dokime::Document::decode(document.encode_to_vec().as_slice()).unwrap(), document);
    let dokime::card::Body::Plot(plot) = document.sections[0].cards[0].body.as_ref().unwrap() else { panic!("plot card") };
    assert_eq!(plot.kind, dokime::PlotKind::HeatStrip as i32);
    assert_eq!(plot.cell, Some(samples.field("cell")));
    assert_eq!(plot.facet, Some(samples.field("facet")));
    assert_eq!(plot.reduction, Some(dokime::Reduction::Mean as i32));
}
