/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Materialize Canonical Template Cases

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_domain::operations::planning::MaterializeTemplate;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[command(flatten)]
    input: super::input::Args,
    #[command(flatten)]
    connection: crate::connection::Args,
    #[arg(long, help = "Emit The Canonical Reply As Compact JSON.")]
    json: bool,

    #[arg(long, conflicts_with = "json", help = "Print Each Materialized Case UUID On Its Own Line.")]
    id_only: bool,
}

/* === Enums === */
enum Input {
    Canonical(MaterializeTemplate),
    Template(super::input::Planned),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> crate::Result<()> {
    // === Read Request === //

    let mut input = args.input;
    let input = match input.input_file.take() {
        Some(path) => {
            let bytes = harmos::filesystem::input::read(&path, 16 * 1024 * 1024)?;
            let request = serde_json::from_slice::<MaterializeTemplate>(&bytes)
                .map_err(|error| crate::Error::Input(format!("invalid plan input: {error}")))?;
            if request.selection.is_none() {
                return Err(crate::Error::Input("one canonical template selection is required".into()));
            }
            Input::Canonical(request)
        }
        None => Input::Template(input.plan()?.single_template()?),
    };

    // === Resolve And Materialize === //

    let client = args.connection.connect().await?;
    let result = async {
        let request = match input {
            Input::Canonical(request) => request,
            Input::Template(planned) => {
                let mut plan = planned.resolve(&client).await?.expect("validated single template");
                MaterializeTemplate { selection: plan.selections.pop() }
            }
        };
        let reply = client.materialize_template(request).await?;
        if args.id_only {
            for case in reply.cases {
                println!("{}", case.case_id);
            }
            return Ok(());
        }
        super::input::print(&client, &reply, args.json).await
    }
    .await;
    client.close_and_wait().await;
    result
}

/* === Private Functions === */

/* === Tests === */
