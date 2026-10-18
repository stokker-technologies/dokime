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
use std::{
    io::Write,
    path::{Path, PathBuf},
};

// Third Party Imports

// Local Imports
use dokime_client::{
    dynamic::{Event, MAX_INPUT_FRAMES},
    operations::MAX_INPUT_BYTES,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(help = "Operation Identifier From `operation list`.")]
    operation: String,
    #[arg(long, help = "Select The Discovered Provider Owner When More Than One Provider Exists.")]
    owner: Option<String>,
    #[command(flatten)]
    connection: crate::connection::Args,
    #[arg(long, conflicts_with = "json", help = "Read Request JSON From This File; Defaults To An Empty Object.")]
    input: Option<PathBuf>,

    #[arg(long, conflicts_with = "input", help = "Supply Inline Request JSON.")]
    json: Option<String>,

    #[arg(long, help = "Atomically Save All Response Events As One JSON Array After Successful Completion.")]
    output: Option<PathBuf>,

    #[arg(long, requires = "output", help = "Replace An Existing Regular Response File After Successful Completion.")]
    overwrite: bool,

    #[arg(long, help = "Read Input Stream Items As One JSON Value Per Line; Bounded To 16 MiB In Total.")]
    input_stream_file: Option<PathBuf>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let input = read_request(args.input.as_deref(), args.json.as_deref())?;
    let items = read_items(args.input_stream_file.as_deref(), MAX_INPUT_BYTES - input.len())?;

    // Reserve Writable Staging Before Any Device Dispatch.
    let staging = args.output.as_deref().map(|path| prepare_output(path, args.overwrite));
    let mut staged = staging.transpose()?;
    let client = args.connection.connect().await?;
    let interrupt = tokio::signal::ctrl_c();
    tokio::pin!(interrupt);
    let dispatch = dispatch(&client, &args, &input, items, &mut staged);
    let result = tokio::select! {
        biased;
        interrupted = &mut interrupt => {
            let reason = interrupted.map_or_else(crate::Error::from, |()| crate::Error::Input("Stopped waiting for the device".into()));
            Err(crate::Error::OperationUncertain { source: Box::new(reason) })
        }
        result = dispatch => result,
    };
    client.close_and_wait().await;
    result?;
    if let (Some(file), Some(destination)) = (staged, args.output) {
        let publication = if args.overwrite {
            harmos::filesystem::publication::replace(file.path(), &destination)
        } else {
            harmos::filesystem::publication::install(file.path(), &destination)
        };
        publication.map_err(std::io::Error::from).map_err(output_error)?;
    }
    Ok(())
}

/* === Private Functions === */
fn read_request(path: Option<&Path>, inline: Option<&str>) -> crate::Result<Vec<u8>> {
    let input = match path {
        Some(path) => harmos::filesystem::input::read(path, MAX_INPUT_BYTES)?,
        None => inline.unwrap_or("{}").as_bytes().to_vec(),
    };
    if input.len() > MAX_INPUT_BYTES {
        return Err(dokime_client::operations::Error::InputTooLarge.into());
    }
    // Reject Malformed JSON Before Selecting Or Opening A Connection.
    let _: serde_json::Value = serde_json::from_slice(&input).map_err(|error| crate::Error::Input(error.to_string()))?;
    Ok(input)
}

fn read_items(path: Option<&Path>, remaining: usize) -> crate::Result<Vec<Vec<u8>>> {
    let Some(path) = path else { return Ok(Vec::new()) };
    let bytes = harmos::filesystem::input::read(path, remaining)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| crate::Error::Input("Input stream must be UTF-8 JSON lines".into()))?;

    let mut items = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if items.len() == MAX_INPUT_FRAMES {
            return Err(dokime_client::operations::Error::InputTooLarge.into());
        }
        let _: serde_json::Value = serde_json::from_str(line).map_err(|error| crate::Error::Input(error.to_string()))?;
        let item = line.as_bytes().to_vec();
        items.push(item);
    }
    Ok(items)
}

async fn dispatch(
    client: &dokime_client::Client, args: &Args, input: &[u8], items: Vec<Vec<u8>>, staged: &mut Option<tempfile::NamedTempFile>,
) -> crate::Result<()> {
    let stream = client.call_operation(&args.operation, args.owner.as_deref(), input, items).await.map_err(dispatch_error)?;
    let result = receive(&stream, staged).await;
    stream.close().await;
    result
}

async fn receive(stream: &dokime_client::ResponseStream<Event>, staged: &mut Option<tempfile::NamedTempFile>) -> crate::Result<()> {
    let mut first = true;
    let mut completed = false;
    if let Some(file) = staged.as_mut() {
        file.write_all(b"[").map_err(output_error)?;
    }

    while let Some(event) = stream.message().await.map_err(|error| dispatch_error(error.into()))? {
        if completed {
            let source = crate::Error::Input("Operation continued after completion".into());
            return Err(crate::Error::OperationUncertain { source: Box::new(source) });
        }
        completed = matches!(&event, Event::Completion(_));
        let value = match event {
            Event::Item(value) => serde_json::json!({"event": "item", "value": value}),
            Event::Completion(value) => serde_json::json!({"event": "completion", "value": value}),
        };
        if let Some(file) = staged.as_mut() {
            write_event(file, &value, &mut first).map_err(output_error)?;
            continue;
        }
        let mut stdout = std::io::stdout().lock();
        serde_json::to_writer(&mut stdout, &value).map_err(output_error)?;
        stdout.write_all(b"\n").map_err(output_error)?;
    }

    if !completed {
        let source = crate::Error::Input("Operation ended without a completion response".into());
        return Err(crate::Error::OperationUncertain { source: Box::new(source) });
    }
    if let Some(file) = staged.as_mut() {
        file.write_all(b"]\n").map_err(output_error)?;
    }
    Ok(())
}

fn prepare_output(path: &Path, overwrite: bool) -> crate::Result<tempfile::NamedTempFile> {
    if path.file_name().is_none() {
        return Err(crate::Error::Input("Response destination must name a file".into()));
    }
    match std::fs::symlink_metadata(path) {
        Ok(_) if !overwrite => return Err(crate::Error::Input("Response destination exists; use --overwrite to replace it".into())),
        Ok(metadata) if !metadata.is_file() || metadata.file_type().is_symlink() => {
            return Err(crate::Error::Input("Response destination must be a regular file".into()));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let parent = path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    Ok(tempfile::NamedTempFile::new_in(parent)?)
}

fn write_event(writer: &mut impl Write, value: &serde_json::Value, first: &mut bool) -> serde_json::Result<()> {
    if !*first {
        writer.write_all(b",").map_err(serde_json::Error::io)?;
    }
    serde_json::to_writer(writer, value)?;
    *first = false;
    Ok(())
}

fn output_error(error: impl Into<crate::Error>) -> crate::Error {
    crate::Error::OperationOutput { source: Box::new(error.into()) }
}

fn dispatch_error(error: dokime_client::operations::Error) -> crate::Error {
    let uncertain = match &error {
        dokime_client::operations::Error::Client(dokime_client::Error::Rpc(status)) => matches!(
            status.code(),
            dokime_client::RpcCode::Cancelled
                | dokime_client::RpcCode::DeadlineExceeded
                | dokime_client::RpcCode::Unknown
                | dokime_client::RpcCode::Internal
                | dokime_client::RpcCode::Unavailable
                | dokime_client::RpcCode::DataLoss
        ),
        _ => false,
    };
    if uncertain { crate::Error::OperationUncertain { source: Box::new(error.into()) } } else { error.into() }
}

/* === Tests === */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_events() {
        let mut bytes = b"[".to_vec();
        let mut first = true;
        let item = serde_json::json!({"event": "item", "value": {"voltage": 230}});
        let completion = serde_json::json!({"event": "completion", "value": {"enabled": true}});
        write_event(&mut bytes, &item, &mut first).unwrap();
        write_event(&mut bytes, &completion, &mut first).unwrap();
        bytes.extend_from_slice(b"]\n");
        assert_eq!(serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(), serde_json::json!([item, completion]));

        let mut bytes = b"[".to_vec();
        let mut first = true;
        write_event(&mut bytes, &completion, &mut first).unwrap();
        bytes.extend_from_slice(b"]\n");
        assert_eq!(serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(), serde_json::json!([completion]));
    }

    #[test]
    fn uncertain_outcome() {
        let error = dokime_client::Error::Rpc(tonic::Status::deadline_exceeded("response lost"));
        let uncertain = dispatch_error(error.into());
        assert!(matches!(uncertain, crate::Error::OperationUncertain { .. }));
        assert_eq!(uncertain.exit_code(), std::process::ExitCode::from(3));

        let error = dokime_client::Error::Rpc(tonic::Status::permission_denied("command refused"));
        let refused = dispatch_error(error.into());
        assert!(matches!(refused, crate::Error::Operation(_)));
    }
}
