# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Own Disposable Directories in Normal System Temporary Storage"

# === Imports === #

# Standard Library Imports
import os
import tempfile
from pathlib import Path
from typing import BinaryIO

# isort: split

# Third Party Imports

# isort: split

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Directory:
    """Own A Locked Temporary Root Until Explicitly Closed."""

    def __init__(self) -> None:
        self._temporary = tempfile.TemporaryDirectory(prefix="project-tmp-")
        self.root = Path(self._temporary.name)
        self.path = self.root / "contents"
        self.owner: BinaryIO | None = None
        try:
            self.owner = (self.root / "owner.lock").open("x+b")
            if os.name == "nt":
                import msvcrt

                msvcrt.locking(self.owner.fileno(), msvcrt.LK_NBLCK, 1)
            else:
                import fcntl

                fcntl.flock(self.owner, fcntl.LOCK_EX | fcntl.LOCK_NB)
            self.owner.write(b"project-temporary-v1\n")
            self.owner.flush()
            self.path.mkdir()
        except BaseException:
            self.close()
            raise

    def close(self) -> None:
        if self.owner is not None:
            self.owner.close()
            self.owner = None
        self._temporary.cleanup()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
