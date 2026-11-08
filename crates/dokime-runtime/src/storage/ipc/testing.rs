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

// Third Party Imports

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Writer(super::Writer);

pub struct Checkpoint(super::Checkpoint);

pub struct BatchIndex(super::BatchIndex);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Checkpoint {
    pub fn end(&self) -> u64 {
        self.0.end
    }
}

impl Writer {
    pub fn create(path: &Path, schema: SchemaRef) -> Result<Self> {
        super::Writer::create(path, schema).map(Self)
    }
    pub fn recover_empty(path: &Path, schema: SchemaRef) -> Result<Self> {
        super::Writer::recover_empty(path, schema).map(Self)
    }
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint(self.0.checkpoint())
    }
    pub fn append(&mut self, batch: &RecordBatch) -> Result<BatchIndex> {
        self.0.append(batch).map(BatchIndex)
    }
    pub fn rollback(&mut self, checkpoint: &Checkpoint) -> Result<()> {
        self.0.rollback(&checkpoint.0)
    }
    pub fn finish(&mut self) -> Result<(u64, [u8; 32])> {
        self.0.finish()
    }
}

/* === Public Functions === */

pub fn read_batch(path: &Path, schema: SchemaRef, index: &BatchIndex) -> Result<RecordBatch> {
    super::read_batch(path, schema, &index.0)
}

pub fn recover(path: &Path, schema: SchemaRef, indices: &[BatchIndex], checkpoint: &Checkpoint) -> Result<(u64, [u8; 32])> {
    let indices = indices.iter().map(|index| index.0.clone()).collect::<Vec<_>>();
    super::recover(path, schema, &indices, &checkpoint.0)
}

/* === Private Functions === */

/* === Tests === */
