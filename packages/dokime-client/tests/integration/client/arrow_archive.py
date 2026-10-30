# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Native Arrow Evidence Archives With Polars"""

# === Imports === #

# Standard Library Imports
import hashlib
import io
import json
import sqlite3
import sys
import tempfile
import zipfile
from contextlib import closing
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


# === Public Functions === #
def verify_arrow_archive(path: Path) -> None:
    """Read Native Exported Batches With An Independent Arrow Implementation."""
    import polars as pl

    with zipfile.ZipFile(path) as archive, tempfile.TemporaryDirectory() as folder:
        database_path = Path(folder) / "evidence.sqlite"
        database_path.write_bytes(archive.read("evidence.sqlite"))
        with closing(sqlite3.connect(database_path)) as database:
            assert database.execute("PRAGMA foreign_key_check").fetchall() == []
            streams = database.execute(
                "SELECT f.path,f.size_bytes,f.sha256,s.rows,s.batches_json,e.execution_id "
                "FROM streams s JOIN files f ON f.id=s.file_id "
                "JOIN executions e ON e.execution_id=s.execution_id WHERE f.kind='telemetry'"
            ).fetchall()
            assert streams
            assert database.execute(
                "SELECT count(*) FROM artifacts a JOIN files f ON f.id=a.file_id WHERE f.kind='telemetry'"
            ).fetchone() == (0,)
            assert database.execute(
                "SELECT count(*) FROM sqlite_schema WHERE name='stream_columns' OR name GLOB 'stream_[0-9]*'"
            ).fetchone() == (0,)
            for member, size, digest, rows, batches_json, execution in streams:
                assert member.endswith(".arrow") and "/telemetry/" in f"/{member}"
                contents = archive.read(member)
                assert len(contents) == size
                assert hashlib.sha256(contents).hexdigest() == digest
                frame = pl.read_ipc(io.BytesIO(contents))
                assert frame.height == rows == sum(batch["rows"] for batch in json.loads(batches_json))
                assert set(frame.columns) == {"_storage", "values"}
                assert frame["values"].struct.fields
                scopes = frame["_storage"].struct.field("scope").to_list()
                assert all(json.loads(scope)["test_execution_id"] == execution for scope in scopes)


# === Private Functions === #

# === Entry Point === #
if __name__ == "__main__":
    verify_arrow_archive(Path(sys.argv[1]))
