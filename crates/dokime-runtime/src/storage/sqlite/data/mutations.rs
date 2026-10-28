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
use std::result::Result;
use std::time::{SystemTime, UNIX_EPOCH};

// Third Party Imports
use harmos::storage::*;
use rusqlite::params;

// Local Imports
use super::super::{Error, State};
use super::{entry, saved};

/* === Constants === */

pub(super) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS storage_mutations (
    window TEXT NOT NULL, key TEXT NOT NULL, expires TEXT NOT NULL,
    body BLOB NOT NULL, PRIMARY KEY(window,key)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS storage_mutation_expiry ON storage_mutations(expires);
";
/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl State {
    pub(super) async fn mutations_save(&self, receipts: StorageBatch<MutationReceipt>) -> Result<(), Error> {
        self.call(move |connection| {
            Ok((|| {
                let transaction = connection.transaction()?;
                let now = expiry(SystemTime::now())?;
                transaction.execute("DELETE FROM storage_mutations WHERE expires<=?1", [&now])?;
                let history = saved(&transaction)?.history;
                for receipt in receipts.as_slice() {
                    let expires = expiry(receipt.expires_at)?;
                    if expires <= now {
                        continue;
                    }
                    // Flushes Repeat The Unexpired Receipt Snapshot. Encode Only New Receipts.
                    let retry = {
                        let mut statement = transaction.prepare("SELECT body FROM storage_mutations WHERE window=?1 AND key=?2")?;
                        let mut rows = statement.query(params![receipt.mutation.window.as_str(), receipt.mutation.key.as_str()])?;
                        if let Some(row) = rows.next()? {
                            let body = super::blob(row, 0)?;
                            let prior: MutationReceipt = serde_json::from_slice(body)?;
                            if &prior != receipt {
                                return Err(Error::Conflict("mutation receipt retry differs"));
                            }
                            true
                        } else {
                            false
                        }
                    };
                    if retry {
                        continue;
                    }
                    match &receipt.value {
                        MutationValue::Cursor(cursor) => {
                            history.check(&cursor.history)?;
                            if cursor.sequence > saved(&transaction)?.sequence {
                                return Err(Error::Data("receipt exceeds saved data frontier"));
                            }
                        }
                        MutationValue::Deleted(file) => {
                            history.check(&file.history)?;
                            let deleted: bool = transaction.query_row(
                                "SELECT EXISTS(SELECT 1 FROM storage_deleted_files WHERE id=?1)",
                                [file.id.as_str()],
                                |row| row.get(0),
                            )?;
                            if !deleted {
                                return Err(Error::Data("deletion receipt precedes durable marker"));
                            }
                        }
                        MutationValue::Producer(producer) => {
                            history.check(&producer.session.history)?;
                            let file = StagedFile {
                                history: producer.session.history.clone(),
                                id: producer.session.id.clone(),
                            };
                            let (generation, _, checkpoint) = super::production::stored_checkpoint(&transaction, &file)?
                                .ok_or(Error::Data("producer receipt precedes durable ownership"))?;
                            let finished_length = super::files::stored_metadata(&transaction, &file.id)?
                                .and_then(|(_, finished)| finished)
                                .map_or(0, |reference| reference.length);
                            if generation < producer.session.generation || checkpoint.length.max(finished_length) < producer.length {
                                return Err(Error::Data("producer receipt precedes durable fence or prefix"));
                            }
                        }
                        MutationValue::Reference(reference) => {
                            history.check(&reference.position.history)?;
                            if reference.position.sequence > saved(&transaction)?.sequence
                                || super::files::stored_metadata(&transaction, &reference.metadata.id)?
                                    .and_then(|(_, finished)| finished)
                                    .as_ref()
                                    != Some(reference.as_ref())
                            {
                                return Err(Error::Data("file receipt precedes durable finish"));
                            }
                        }
                        MutationValue::Progress(_) | MutationValue::Rejected => {}
                    }
                    let encoded = entry::encode(&receipt)?;
                    transaction.execute(
                        "INSERT INTO storage_mutations VALUES (?1,?2,?3,?4)",
                        params![receipt.mutation.window.as_str(), receipt.mutation.key.as_str(), expires, encoded],
                    )?;
                }
                super::super::commit(transaction)?;
                Ok(())
            })())
        })
        .await?
    }

    pub(super) async fn mutations_recover(&self, limits: ReadLimits) -> Result<Vec<MutationReceipt>, Error> {
        self.call(move |connection| {
            let result = (|| {
                let now = expiry(SystemTime::now())?;
                // The Newest Unexpired Receipts That Fit; Older Ones Are Omitted.
                let mut statement = connection
                    .prepare("SELECT window,key,expires,body FROM storage_mutations WHERE expires>?1 ORDER BY expires DESC,window,key")?;
                let mut rows = statement.query([now])?;
                let mut receipts = Vec::new();
                let mut bytes = 0usize;
                while receipts.len() < limits.rows.get()
                    && let Some(row) = rows.next()?
                {
                    let body = super::blob(row, 3)?;
                    let receipt: MutationReceipt = serde_json::from_slice(body)?;
                    if receipt.mutation.window.as_str() != super::text(row, 0)?
                        || receipt.mutation.key.as_str() != super::text(row, 1)?
                        || expiry(receipt.expires_at)? != super::text(row, 2)?
                    {
                        return Err(Error::Corrupt("receipt identity index differs"));
                    }
                    match bytes.checked_add(body.len()).filter(|bytes| *bytes <= limits.bytes.get()) {
                        Some(total) => bytes = total,
                        None => break,
                    }
                    receipts.try_reserve_exact(1).map_err(std::io::Error::other)?;
                    receipts.push(receipt);
                }
                Ok(receipts)
            })();
            Ok(result)
        })
        .await?
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn expiry(at: SystemTime) -> Result<String, Error> {
    let value = at.duration_since(UNIX_EPOCH).map_err(|_| Error::Data("receipt expiry precedes epoch"))?.as_nanos();
    let mut encoded = String::new();
    encoded.try_reserve_exact(32).map_err(std::io::Error::other)?;
    use std::fmt::Write as _;
    write!(encoded, "{value:032x}").expect("writing to a string");
    Ok(encoded)
}

/* === Tests === */
