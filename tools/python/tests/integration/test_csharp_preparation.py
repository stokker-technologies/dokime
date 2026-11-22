# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify the Pinned C# Source and Conformance Input Selection."""

# === Imports === #

# Standard Library Imports
import runpy
import shutil
import tarfile
import tempfile
import unittest
from io import BytesIO
from pathlib import Path
from unittest import mock

# isort: split

# Third Party Imports

# isort: split

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class CsharpPreparationTests(unittest.TestCase):
    def test_csharp_preparation_keeps_conformance_inputs_without_library_protocols(self) -> None:
        scratch = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, scratch)
        source = Path(__file__).resolve().parents[4] / "sdk/csharp/tools/prepare_harmos.py"
        script = scratch / "sdk/csharp/tools/prepare_harmos.py"
        script.parent.mkdir(parents=True)
        shutil.copy2(source, script)
        (scratch / "Cargo.toml").write_text('[workspace.dependencies.harmos]\nrev = "pinned"\n')
        (script.parent / "Harmos.csproj").write_text("<Project />")
        contents = {
            "sdk/csharp/src/Harmos/Library.cs": b"library source",
            "sdk/csharp/tests/Harmos.Conformance/Fixture.cs": b"conformance source",
            "sdk/csharp/Directory.Build.props": b"<Project />",
            "sdk/tests/fixtures/conformance/proto/conformance/model/v1/fixture.proto": b"fixture contract",
            "proto/harmos/model/v1/value.proto": b"library contract",
        }
        data = BytesIO()
        archive = tarfile.open(fileobj=data, mode="w:gz")
        try:
            for name, content in contents.items():
                member = tarfile.TarInfo("library/" + name)
                member.size = len(content)
                archive.addfile(member, BytesIO(content))
        finally:
            archive.close()
        data.seek(0)
        with (
            mock.patch.dict("os.environ", {"HARMOS_SOURCE": ""}),
            mock.patch("sys.argv", [str(script), "dotnet", "--configuration", "Debug"]),
            mock.patch("urllib.request.urlopen", return_value=data),
            mock.patch("subprocess.run"),
        ):
            runpy.run_path(str(script), run_name="__main__")
        output = scratch / "target/cs/h"
        for name, content in contents.items():
            if name == "sdk/csharp/Directory.Build.props":
                continue
            if name.startswith("proto/"):
                self.assertFalse((output / name).exists())
                continue
            self.assertEqual((output / name).read_bytes(), content)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
