# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build Python Bindings From The Authoritative Protocol Sources."""

# === Imports === #

import argparse
import hashlib
import json
import shutil
import tempfile
from pathlib import Path

import grpc_tools
from grpc_tools import protoc

# === Public Functions === #


def main(*, check: bool = False) -> None:
    sdk = Path(__file__).resolve().parents[1]
    root = sdk.parents[1]
    protocols = sdk / "proto" if (sdk / "proto").is_dir() else root / "proto"
    source = sdk / "src/dokime"
    output = Path(tempfile.mkdtemp(prefix="dokime-protocol-"))
    try:
        arguments = [
            "protoc",
            f"-I{protocols}",
            f"-I{Path(grpc_tools.__file__).parent / '_proto'}",
            f"--python_out={output}",
            f"--pyi_out={output}",
            f"--grpc_python_out={output}",
            f"--descriptor_set_out={output / 'surface.pb'}",
            "--include_imports",
            *map(str, sorted(protocols.glob("dokime/**/*.proto"))),
        ]
        if protoc.main(arguments):
            raise RuntimeError("Generate Python Protocols: protoc Failed")
        if not check:
            for old in source.rglob("*_pb2*"):
                if old.is_file():
                    old.unlink()
        drift: list[str] = []
        for path in (output / "dokime").rglob("*"):
            if not path.is_file():
                continue
            target = source / "_wire" / path.relative_to(output / "dokime")
            if not check:
                target.parent.mkdir(parents=True, exist_ok=True)
            text = path.read_text(encoding="utf-8")
            for owner in ("dokime", "harmos"):
                for executor in ("model", "host", "plugin"):
                    text = text.replace(f"from {owner}.{executor}", f"from {owner}._wire.{executor}")
                    text = text.replace(f"import {owner}.{executor}.", f"import {owner}._wire.{executor}.")
                    text = text.replace(
                        f"BuildTopDescriptorsAndMessages(DESCRIPTOR, '{owner}.{executor}.",
                        f"BuildTopDescriptorsAndMessages(DESCRIPTOR, '{owner}._wire.{executor}.",
                    )
            if check:
                if not target.is_file() or target.read_text(encoding="utf-8") != text:
                    drift.append(str(target.relative_to(sdk)))
            else:
                target.write_text(text, encoding="utf-8", newline="\n")
                (target.parent / "py.typed").touch()
                (target.parent / "__init__.py").touch(exist_ok=True)
        if check:
            if (output / "surface.pb").read_bytes() != (source / "_wire/surface.pb").read_bytes():
                drift.append("_wire/surface.pb")
        else:
            shutil.copyfile(output / "surface.pb", source / "_wire/surface.pb")
        manifest = {p.relative_to(protocols).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in protocols.rglob("*.proto")}
        text = json.dumps(manifest, sort_keys=True) + "\n"
        if check:
            if (source / "_wire/sources.json").read_text(encoding="utf-8") != text:
                drift.append("_wire/sources.json")
            if drift:
                raise ValueError("Check Python Generation: Frozen Output Drift: " + ", ".join(drift))
        else:
            (source / "_wire/sources.json").write_text(text, encoding="utf-8", newline="\n")
    finally:
        shutil.rmtree(output)


# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    main(check=parser.parse_args().check)
