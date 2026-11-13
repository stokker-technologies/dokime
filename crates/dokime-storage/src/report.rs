/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Published Execution Reports

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;
use rusqlite::params;
use sha2::{Digest, Sha256};

// Local Imports
use crate::{Error, Result, Store, store};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Message)]
pub struct Report {
    #[prost(string, tag = "1")]
    pub execution: String,

    #[prost(string, tag = "2")]
    pub identifier: String,

    #[prost(bytes = "vec", tag = "3")]
    pub execution_seal: Vec<u8>,

    #[prost(message, optional, tag = "4")]
    pub document: Option<v1::Document>,

    #[prost(bytes = "vec", tag = "5")]
    pub html: Vec<u8>,

    #[prost(bytes = "vec", tag = "6")]
    pub seal: Vec<u8>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn plot_preferences(&self, execution: &str, document: &str, plot: &str) -> Result<Option<(Option<u32>, i32)>> {
        use rusqlite::OptionalExtension;
        Ok(self
            .lock()?
            .query_row(
                "SELECT points,technique FROM plot_preferences WHERE execution=?1 AND document=?2 AND plot=?3",
                params![execution, document, plot],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?)
    }
    pub fn save_plot_preferences(&self, execution: &str, document: &str, plot: &str, points: Option<u32>, technique: i32) -> Result<()> {
        self.metadata(execution)?;
        self.lock()?.execute("INSERT INTO plot_preferences VALUES(?1,?2,?3,?4,?5) ON CONFLICT(execution,document,plot) DO UPDATE SET points=excluded.points,technique=excluded.technique",params![execution,document,plot,points,technique])?;
        Ok(())
    }
    pub fn publish_report(&self, execution: &str, execution_seal: &[u8], document: v1::Document, html: Vec<u8>) -> Result<Report> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let (state, seal): (i32, Vec<u8>) =
            transaction.query_row("SELECT state,seal FROM executions WHERE id=?1", [execution], |row| Ok((row.get(0)?, row.get(1)?)))?;
        if state != 2 || seal != execution_seal || store::digest(&transaction, execution)? != seal {
            return Err(Error::Invalid("Publish Report: Render a Fixed Snapshot of This Sealed Execution".into()));
        }
        let mut report = Report {
            execution: execution.into(),
            execution_seal: seal,
            document: Some(document),
            html,
            ..Default::default()
        };
        // Seal the exact serialized document, including map entry order, once.
        let mut payload = report.encode_to_vec();
        report.seal = report_digest(&payload)?;
        report.identifier = report.seal.iter().map(|byte| format!("{byte:02x}")).collect();
        prost::encoding::string::encode(2, &report.identifier, &mut payload);
        prost::encoding::bytes::encode(6, &report.seal, &mut payload);
        transaction.execute(
            "INSERT INTO reports VALUES (?1,?2,?3) ON CONFLICT(execution,id) DO NOTHING",
            params![execution, report.identifier, payload],
        )?;
        transaction.commit()?;
        Ok(report)
    }
    pub fn reports(&self, execution: &str) -> Result<Vec<Report>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT payload FROM reports WHERE execution=?1 ORDER BY id")?;
        let bytes = statement.query_map([execution], |row| row.get::<_, Vec<u8>>(0))?.collect::<std::result::Result<Vec<_>, _>>()?;
        bytes
            .iter()
            .map(|bytes| {
                let report = Report::decode(bytes.as_slice())?;
                let identifier: String = report.seal.iter().map(|byte| format!("{byte:02x}")).collect();
                if report.execution != execution || report.seal != report_digest(bytes)? || report.identifier != identifier {
                    return Err(Error::Invalid("Read Report: Restore a Valid Backup of the Published Report".into()));
                }
                let seal: Vec<u8> = connection.query_row("SELECT seal FROM executions WHERE id=?1", [execution], |row| row.get(0))?;
                if report.execution_seal != seal || store::digest(&connection, execution)? != seal {
                    return Err(Error::Invalid("Read Report: Restore the Execution Matching the Published Snapshot".into()));
                }
                Ok(report)
            })
            .collect()
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn report_digest(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut hash = Sha256::new();
    let mut remaining = bytes;
    while !remaining.is_empty() {
        let start = bytes.len() - remaining.len();
        let (tag, wire) = prost::encoding::decode_key(&mut remaining)?;
        prost::encoding::skip_field(wire, tag, &mut remaining, prost::encoding::DecodeContext::default())?;
        if tag != 2 && tag != 6 {
            hash.update(&bytes[start..bytes.len() - remaining.len()]);
        }
    }
    Ok(hash.finalize().to_vec())
}

/* === Tests === */
