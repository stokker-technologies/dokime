/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod zstd_file_round_trip_and_metadata_only_recovery;

mod finalize_appends_only_metadata_and_is_idempotent;

mod rollback_restores_hash_and_block_list;

mod uncataloged_first_flush_can_resume_from_its_verified_header;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use arrow_array::{Int64Array, RecordBatch, StringArray};

use arrow_schema::{DataType, Field, Schema};

use dokime_runtime::testing::ipc::{Writer, read_batch, recover};

use sha2::{Digest, Sha256};

use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::Arc,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn batch() -> RecordBatch {
    let fields = vec![Field::new("integer", DataType::Int64, true), Field::new("text", DataType::Utf8, true)];
    let schema = Arc::new(Schema::new(fields));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![Some(-5), None, Some(i64::MAX)])),
            Arc::new(StringArray::from(vec![Some("hi"), Some("世界"), None])),
        ],
    )
    .unwrap()
}

/* === Tests === */
