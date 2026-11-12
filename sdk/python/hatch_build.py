# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build Python Bindings From The Authoritative Protocol Sources."""

# === Imports === #

# Standard Library Imports
import hashlib
import json
import tempfile
from pathlib import Path

# Third Party Imports
import grpc_tools
from grpc_tools import protoc
from hatchling.builders.hooks.plugin.interface import BuildHookInterface

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class CustomBuildHook(BuildHookInterface):
    def initialize(self, version, build_data):
        root = Path(self.root)
        sources = root / "proto" if (root / "proto").is_dir() else root.parents[1] / "proto"
        self.scratch = tempfile.TemporaryDirectory(prefix="dokime-protobuf-")
        output = Path(self.scratch.name)
        definitions = sorted(sources.glob("dokime/v1/*.proto"))
        if not definitions:
            raise ValueError("Dokime builds require the authoritative proto/dokime/v1 sources")
        arguments = [
            "protoc",
            f"-I{sources}",
            f"-I{Path(grpc_tools.__file__).parent / '_proto'}",
            f"--python_out={output}",
            f"--pyi_out={output}",
            f"--grpc_python_out={output}",
            f"--descriptor_set_out={output / 'surface.pb'}",
            "--include_imports",
            *map(str, definitions),
        ]
        if protoc.main(arguments):
            raise RuntimeError("Dokime protocol generation failed")
        package = output / "dokime/v1"
        (output / "surface.pb").replace(package / "surface.pb")
        manifest = {path.relative_to(sources).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest() for path in definitions}
        manifest["harmos/v1/plugin.proto"] = hashlib.sha256((sources / "harmos/v1/plugin.proto").read_bytes()).hexdigest()
        (package / "sources.json").write_text(json.dumps(manifest, sort_keys=True) + "\n", encoding="utf-8")
        build_data["force_include"][str(package)] = "dokime/v1"
        for name in ("LICENSE", "NOTICE"):
            source = root / name if (root / name).is_file() else root.parents[1] / name
            build_data["force_include"][str(source)] = f"dokime/{name}"
        build_data["pure_python"] = True
        build_data["infer_tag"] = False
        build_data["tag"] = "py3-none-any"

    def finalize(self, version, build_data, artifact_path):
        self.scratch.cleanup()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
