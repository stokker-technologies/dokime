# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build Python Bindings From The Authoritative Protocol Sources."""

# === Imports === #

import hashlib
import json
import shutil
import tempfile
from pathlib import Path

import grpc_tools
from grpc_tools import protoc

# === Public Functions === #


def main() -> None:
    sdk = Path(__file__).resolve().parents[1]
    root = sdk.parents[1]
    protocols = sdk / "proto" if (sdk / "proto").is_dir() else root / "proto"
    source = sdk / "src/dokime"
    with tempfile.TemporaryDirectory(prefix="dokime-protocol-") as scratch:
        output = Path(scratch)
        arguments = [
            "protoc",
            f"-I{protocols}",
            f"-I{Path(grpc_tools.__file__).parent / '_proto'}",
            f"--python_out={output}",
            f"--mypy_out=quiet:{output}",
            f"--mypy_grpc_out=quiet:{output}",
            f"--grpc_python_out={output}",
            f"--descriptor_set_out={output / 'surface.pb'}",
            "--include_imports",
            *map(str, sorted(protocols.glob("dokime/**/*.proto"))),
        ]
        if protoc.main(arguments):
            raise RuntimeError("Generate Python Protocols: protoc Failed")
        for old in source.rglob("*_pb2*"):
            if old.is_file():
                old.unlink()
        for path in (output / "dokime").rglob("*"):
            if not path.is_file():
                continue
            target = source / path.relative_to(output / "dokime")
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, target)
            (target.parent / "py.typed").touch()
            (target.parent / "__init__.py").touch(exist_ok=True)
        shutil.copyfile(output / "surface.pb", source / "v1/surface.pb")
        manifest = {p.relative_to(protocols).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in protocols.rglob("*.proto")}
        (source / "v1/sources.json").write_text(json.dumps(manifest, sort_keys=True) + "\n", encoding="utf-8", newline="\n")


# === Entry Point === #

if __name__ == "__main__":
    main()
