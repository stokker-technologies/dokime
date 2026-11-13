/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Discover and Invoke Host Operations

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports


/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(clap::Subcommand)]
pub(crate) enum Command {
    List {
        #[arg(long)]
        json: bool,
    },
}

/* === Traits === */

/* === Implementations === */

impl Command {
    pub fn action(&self) -> &'static str {
        match self {
            Self::List { .. } => "List Operations",
        }
    }
}

/* === Public Functions === */

pub(crate) async fn run(command: Command, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let channel = settings.channel(endpoint, false).await?;
    let client = dokime::Client::from_channel(channel, settings.client())?;
    match command {
        Command::List { json } => {
            let request = dokime::client::CatalogEntriesListRequest {
                kind: dokime::client::CatalogKind::Operations as i32,
                ..Default::default()
            };
            let reply = client.catalog(request).await?.into_inner();
            crate::output::print(&reply, json)
        }
    }
}

/* === Private Functions === */

/* === Tests === */
