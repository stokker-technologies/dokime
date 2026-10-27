/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A directory on an FTP server. `ftps://` secures the control and data
// channels with explicit TLS verified by the platform's roots; plain `ftp://`
// is accepted only on a bench the runtime profile marks trusted. Discovery
// lists directories (MLSD, or LIST where MLSD is refused); ranged reads use
// REST. An interrupted command drops the connection, and the next pass
// reconnects.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::VecDeque, future::Future, sync::Arc, time::Duration};

// Third Party Imports
use suppaftp::{
    FtpError,
    list::{File as Listed, ListParser},
    tokio::{AsyncRustlsConnector, AsyncRustlsFtpStream},
    tokio_rustls::{TlsConnector, rustls},
    types::FileType,
};
use tokio::io::AsyncReadExt;

// Local Imports
use super::source::{Entry, Listing, ReadFailure};
use super::{UNSUPPORTED, UNTRUSTED, capped, invalid, unreachable_source};
use crate::files::FileMatcher;
use crate::{runtime::Error, runtime::Result};

/* === Constants === */

const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const TRANSFER_TIMEOUT: Duration = Duration::from_secs(300);
const SCAN_ENTRIES: usize = 100_000;
const DEFAULT_PORT: u16 = 21;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(in crate::domains::files) struct FtpSource {
    endpoint: Endpoint,
    connection: Option<AsyncRustlsFtpStream>,
    listing: ListingCommand,
}

#[derive(Clone)]
struct Endpoint {
    host: String,
    port: u16,
    secure: bool,
    user: String,
    password: String,
    // An absolute remote directory without a trailing slash, or `/`.
    root: String,
}

/* === Enums === */

#[derive(Clone, Copy, PartialEq, Eq)]
enum ListingCommand {
    Mlsd,
    List,
}

/* === Traits === */

/* === Implementations === */

impl FtpSource {
    pub(super) async fn open(url: &str, password: Option<String>, trusted: bool) -> Result<Self> {
        let endpoint = Endpoint::parse(url, password)?;
        if !endpoint.secure && !trusted {
            return Err(Error::conflict(UNTRUSTED, "plain FTP is accepted only on a bench the runtime profile marks trusted; use ftps://"));
        }
        let mut source = Self { endpoint, connection: None, listing: ListingCommand::Mlsd };

        // Fail Fast On An Unreachable Server Or Refused Login
        let connection = source.connection().await?;
        source.connection = Some(connection);
        Ok(source)
    }

    pub(super) async fn scan(&mut self, matcher: &FileMatcher) -> Result<Listing> {
        let mut connection = self.connection().await?;
        let mut listing = Listing::new();
        let mut directories = VecDeque::from([String::new()]);
        let mut visited = 0_usize;
        while let Some(directory) = directories.pop_front() {
            let remote = self.endpoint.remote(&directory);
            let lines = self.list(&mut connection, &remote).await?;
            for line in lines {
                visited += 1;
                if visited > SCAN_ENTRIES {
                    return Err(capped(format!("the watch root holds more than {} entries", SCAN_ENTRIES)));
                }
                let Some(listed) = self.parse(&line) else {
                    continue;
                };
                let name = listed.name();
                if name.is_empty() || name == "." || name == ".." || name.contains('/') {
                    continue;
                }
                let relative = if directory.is_empty() { name.to_owned() } else { format!("{directory}/{name}") };
                if listed.is_directory() {
                    directories.push_back(relative);
                } else if listed.is_file() && matcher.matches(&relative) {
                    listing.insert(relative, Entry { size: listed.size() as u64, modified: Some(listed.modified()), identity: None });
                }
            }
        }
        self.connection = Some(connection);
        Ok(listing)
    }

    pub(super) async fn read_range(
        &mut self, relative: &str, offset: u64, length: u64, sink: &mut tokio::fs::File,
    ) -> std::result::Result<u64, ReadFailure> {
        let mut connection = self.connection().await?;
        let remote = self.endpoint.remote(relative);

        // Resume From The Committed Offset; A Refusal Leaves The Session Usable
        if offset > 0 {
            let start = usize::try_from(offset).map_err(|_| ReadFailure::Failed(invalid("the resume offset overflows")))?;
            match tokio::time::timeout(COMMAND_TIMEOUT, connection.resume_transfer(start)).await {
                Ok(Ok(())) => {}
                Ok(Err(FtpError::UnexpectedResponse(response)))
                    if matches!(
                        response.status,
                        suppaftp::Status::BadCommand
                            | suppaftp::Status::BadArguments
                            | suppaftp::Status::NotImplemented
                            | suppaftp::Status::NotImplementedParameter
                    ) =>
                {
                    self.connection = Some(connection);
                    return Err(ReadFailure::ResumeUnsupported);
                }
                Ok(Err(FtpError::UnexpectedResponse(response))) => {
                    return Err(ReadFailure::Failed(invalid(format!(
                        "FTP resume was refused for {relative:?}: {response}; verify permissions and source path"
                    ))));
                }
                Ok(Err(source)) => {
                    return Err(ReadFailure::Failed(unreachable_source(format!("{relative}: {source}"))));
                }
                Err(_) => {
                    return Err(ReadFailure::Failed(unreachable_source(format!("{relative}: the resume request timed out"))));
                }
            }
        }

        // Copy At Most The Listed Length, Then Complete The Transfer
        let mut stream = match tokio::time::timeout(COMMAND_TIMEOUT, connection.retr_as_stream(&remote)).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(FtpError::UnexpectedResponse(response))) => {
                return Err(ReadFailure::Failed(invalid(format!(
                    "FTP retrieval was refused for {relative:?}: {response}; verify permissions and source path"
                ))));
            }
            Ok(Err(source)) => return Err(ReadFailure::Failed(unreachable_source(source))),
            Err(_) => return Err(ReadFailure::Failed(unreachable_source("FTP retrieval setup timed out"))),
        };
        let copied = tokio::time::timeout(TRANSFER_TIMEOUT, async { tokio::io::copy(&mut (&mut stream).take(length), sink).await })
            .await
            .map_err(|_| unreachable_source(format!("{relative}: the transfer timed out")))?
            .map_err(|source| unreachable_source(format!("{relative}: {source}")))?;
        match tokio::time::timeout(COMMAND_TIMEOUT, stream.finish()).await {
            Ok(Ok(())) => {}
            // Closing Early After The Listed Length May Draw An Abort Reply
            Ok(Err(FtpError::UnexpectedResponse(_))) if copied == length => {}
            Ok(Err(source)) => {
                return Err(ReadFailure::Failed(unreachable_source(format!("{relative}: {source}"))));
            }
            Err(_) => {
                return Err(ReadFailure::Failed(unreachable_source(format!("{relative}: the transfer did not complete"))));
            }
        }
        self.connection = Some(connection);
        Ok(copied)
    }

    // Takes the open session, or connects and logs in afresh.
    async fn connection(&mut self) -> Result<AsyncRustlsFtpStream> {
        if let Some(connection) = self.connection.take() {
            return Ok(connection);
        }
        let endpoint = &self.endpoint;
        let connection = bounded(COMMAND_TIMEOUT, AsyncRustlsFtpStream::connect((endpoint.host.as_str(), endpoint.port))).await?;
        let mut connection = if endpoint.secure {
            bounded(COMMAND_TIMEOUT, connection.into_secure(tls_connector()?, &endpoint.host)).await?
        } else {
            connection
        };
        bounded(COMMAND_TIMEOUT, connection.login(endpoint.user.as_str(), endpoint.password.as_str())).await?;
        bounded(COMMAND_TIMEOUT, connection.transfer_type(FileType::Binary)).await?;
        Ok(connection)
    }

    async fn list(&mut self, connection: &mut AsyncRustlsFtpStream, remote: &str) -> Result<Vec<String>> {
        if self.listing == ListingCommand::Mlsd {
            match tokio::time::timeout(COMMAND_TIMEOUT, connection.mlsd(Some(remote))).await {
                Ok(Ok(lines)) => return Ok(lines),
                // A Server Without MLSD Refuses It; Fall Back To LIST
                Ok(Err(FtpError::UnexpectedResponse(_))) => self.listing = ListingCommand::List,
                Ok(Err(source)) => return Err(unreachable_source(source)),
                Err(_) => return Err(unreachable_source("the directory listing timed out")),
            }
        }
        match tokio::time::timeout(COMMAND_TIMEOUT, connection.list(Some(remote))).await {
            Ok(Ok(lines)) => Ok(lines),
            Ok(Err(FtpError::UnexpectedResponse(response))) if response.status == suppaftp::Status::FileUnavailable => Err(invalid(
                "FTP directory unavailable (550): it may be absent or permission denied. Create the directory and grant listing access before watching; automatic root waiting cannot distinguish these causes",
            )),
            Ok(Err(source)) => Err(unreachable_source(source)),
            Err(_) => Err(unreachable_source("the directory listing timed out")),
        }
    }

    fn parse(&self, line: &str) -> Option<Listed> {
        match self.listing {
            ListingCommand::Mlsd => ListParser::parse_mlsd(line).ok(),
            ListingCommand::List => ListParser::parse_posix(line).or_else(|_| ListParser::parse_dos(line)).ok(),
        }
    }
}

impl Endpoint {
    // `ftps://[user@]host[:port]/root` or `ftp://…`; a password in the URL is
    // refused because it belongs in the secret store.
    fn parse(url: &str, password: Option<String>) -> Result<Self> {
        let (scheme, rest) = url.split_once("://").ok_or_else(|| invalid("an FTP source needs a URL with a scheme"))?;
        let secure = match scheme.to_ascii_lowercase().as_str() {
            "ftps" => true,
            "ftp" => false,
            "sftp" => {
                return Err(Error::conflict(UNSUPPORTED, "SFTP sources are not supported in this build yet"));
            }
            _ => return Err(invalid("an FTP source URL must use ftps:// or ftp://")),
        };
        let (authority, root) = match rest.find('/') {
            Some(index) => rest.split_at(index),
            None => (rest, "/"),
        };
        if authority.is_empty() || url.contains(['%', '?', '#']) || url.chars().any(char::is_control) {
            return Err(invalid("an FTP source URL needs a host and no escapes, query or fragment"));
        }
        let (user, host_port) = match authority.rsplit_once('@') {
            Some((user, _)) if user.contains(':') => {
                return Err(invalid("an FTP source URL must not carry a password; name a secret instead"));
            }
            Some((user, host_port)) => (Some(user.to_owned()), host_port),
            None => (None, authority),
        };
        let (host, port) = split_host_port(host_port)?;
        let root = root.trim_end_matches('/');
        let root = if root.is_empty() { "/" } else { root };
        if root.split('/').any(|segment| segment == "..") {
            return Err(invalid("an FTP source root must not contain '..'"));
        }
        let anonymous = user.is_none();
        Ok(Self {
            host,
            port,
            secure,
            user: user.unwrap_or_else(|| "anonymous".to_owned()),
            password: password.unwrap_or_else(|| if anonymous { "anonymous@".to_owned() } else { String::new() }),
            root: root.to_owned(),
        })
    }

    fn remote(&self, relative: &str) -> String {
        match (self.root.as_str(), relative) {
            (root, "") => root.to_owned(),
            ("/", relative) => format!("/{relative}"),
            (root, relative) => format!("{root}/{relative}"),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn split_host_port(value: &str) -> Result<(String, u16)> {
    // A Bracketed IPv6 Literal Keeps Its Colons
    let (host, port) = if let Some(rest) = value.strip_prefix('[') {
        let (host, rest) = rest.split_once(']').ok_or_else(|| invalid("an FTP source host is malformed"))?;
        (host, rest.strip_prefix(':'))
    } else {
        match value.rsplit_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (value, None),
        }
    };
    if host.is_empty() {
        return Err(invalid("an FTP source URL needs a host"));
    }
    let port = match port {
        Some(port) => port.parse::<u16>().ok().filter(|port| *port != 0).ok_or_else(|| invalid("an FTP source port is invalid"))?,
        None => DEFAULT_PORT,
    };
    Ok((host.to_owned(), port))
}

fn tls_connector() -> Result<AsyncRustlsConnector> {
    use rustls_platform_verifier::BuilderVerifierExt;
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .and_then(|builder| builder.with_platform_verifier())
        .map_err(|source| unreachable_source(format!("FTPS configuration: {source}")))?
        .with_no_client_auth();
    Ok(AsyncRustlsConnector::from(TlsConnector::from(Arc::new(config))))
}

// Bounds one FTP command; a refusal or timeout means the source is unreachable.
async fn bounded<T>(timeout: Duration, command: impl Future<Output = suppaftp::FtpResult<T>>) -> Result<T> {
    match tokio::time::timeout(timeout, command).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(source)) => Err(unreachable_source(source)),
        Err(_) => Err(unreachable_source("the FTP command timed out")),
    }
}

/* === Tests === */
