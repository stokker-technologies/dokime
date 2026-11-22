/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Retain Exact SDK Publication Boundaries

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::HashMap, error::Error};

// Third Party Imports

// Local Imports
use super::support;
use dokime_protocol::model::{Value, value};
use dokime_runtime::Host;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn publication_boundaries() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        let root = support::tempdir()?;
        let host = Host::open(root.path(), support::settings(), support::service_settings()).await?;
        let entry = host.add_managed("boundary", &support::boundary_fixture()?, &[]).await?;
        let mut retained = Vec::new();
        for rows in [1023, 1024, 1025] {
            let origin = host
                .materialize(
                    &entry,
                    "execution.execution.boundary",
                    HashMap::from([(
                        "rows".into(),
                        Value {
                            kind: Some(value::Kind::UnsignedInteger(rows)),
                        },
                    )]),
                )
                .await?;
            let cycle = host.start_cycle(vec![origin]).await?.wait().await?;
            let samples = &cycle.executions[0].samples;
            retained.push(samples.len() as u64);
            for (index, sample) in samples.iter().enumerate() {
                assert_eq!(sample.sequence, index as u64 + 1);
                let reading = <support::Reading as prost::Message>::decode(sample.row.as_slice())?;
                assert_eq!(reading.value, index as f64);
            }
            assert!(samples.windows(2).all(|pair| pair[1].source_ns - pair[0].source_ns == 1_000_000));
        }
        assert_eq!(retained, [1023, 1024, 1025], "Seal SDK Publication: Retain the Requested Row Counts");
        host.shutdown().await?;
        Ok(())
    }))
}
