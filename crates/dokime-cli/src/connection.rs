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
use std::{path::PathBuf, time::Duration};

// Third Party Imports
use zeroize::Zeroizing;

// Local Imports
use dokime::client::{Client, Target};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
#[group(id = "connection")]
pub(crate) struct Args {
    #[arg(
        long,
        help = "Backend HTTP(S) Origin; Plain HTTP Is Restricted To Loopback."
    )]
    endpoint: String,
    #[arg(long, help = "Read A Bearer Token From This Local File.")]
    token_file: Option<PathBuf>,
    #[arg(
        long,
        help = "Read An Additional PEM CA Certificate From This Local File."
    )]
    ca: Option<PathBuf>,
    #[arg(long, help = "Require This Backend Identity During Connection.")]
    expected_backend: Option<String>,
    #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..=86400), help = "Connection And Call Deadline In Seconds.")]
    timeout_seconds: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Args {
    pub(crate) async fn connect(&self) -> crate::Result<Client> {
        let ca = self
            .ca
            .as_deref()
            .map(|path| {
                String::from_utf8(crate::input::read(path, 1024 * 1024)?)
                    .map_err(|_| crate::Error::Input("CA certificate must be UTF-8 PEM".into()))
            })
            .transpose()?;
        let target = Target::new(&self.endpoint, ca)?;
        let token = self
            .token_file
            .as_deref()
            .map(|path| {
                let bytes = Zeroizing::new(crate::input::read(path, 4098)?);
                let text = std::str::from_utf8(&bytes)
                    .map_err(|_| crate::Error::Input("bearer token must be UTF-8 text".into()))?;
                Ok::<_, crate::Error>(Zeroizing::new(text.trim().to_owned()))
            })
            .transpose()?;
        let timeout = Duration::from_secs(self.timeout_seconds);
        let connect = Client::connect(
            target,
            token.as_deref().map(String::as_str),
            self.expected_backend.as_deref(),
        );
        let client = tokio::time::timeout(timeout, connect)
            .await
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::TimedOut, error))??;
        Ok(client.with_timeout(timeout))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
