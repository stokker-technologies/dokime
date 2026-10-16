# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Select Contract Exports From Cargo Build Messages"""

# === Imports === #

# Standard Library Imports
import hashlib
import io
import json
import tarfile
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from dokime_tools.bindings import _normalize, generate

# Third Party Imports
# Local Imports
from dokime_tools.contracts import archive, locate, restore, restored

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ContractExportTests(unittest.TestCase):
    def test_selected_build(self):
        with TemporaryDirectory() as temporary:
            directory = Path(temporary)
            output = directory / "selected" / "grpc" / "operations"
            output.mkdir(parents=True)
            (output / "descriptors.bin").write_bytes(b"selected descriptors")
            (output / "contracts.json").write_text("[]")
            (output / "service.proto").write_text('syntax = "proto3";')
            manifest = output / "manifest.txt"
            manifest.write_text("descriptors.bin\ncontracts.json\nservice.proto\n")
            messages = directory / "cargo.jsonl"
            package = "path+file:///selected/dokime-contract#0.1.0"
            script = {
                "reason": "build-script-executed",
                "package_id": package,
                "out_dir": str(output.parents[1]),
            }
            artifact = {
                "reason": "compiler-artifact",
                "package_id": package,
                "target": {"name": "dokime-contract-export", "kind": ["bin"]},
                "features": [],
            }
            finished = {"reason": "build-finished", "success": True}
            events = [script, artifact, finished]

            messages.write_text("\n".join(map(json.dumps, events)))
            self.assertEqual(locate(messages), output)

            unrelated = [
                {
                    **script,
                    "package_id": "dependency",
                    "out_dir": str(directory / variant),
                }
                for variant in ("host", "target")
            ]
            messages.write_text("\n".join(map(json.dumps, unrelated + events)))
            self.assertEqual(locate(messages), output)

            refused = {
                "failed build": [script, artifact, {**finished, "success": False}],
                "concatenated builds": events + events,
                "missing selected producer": [artifact, finished],
                "ambiguous producer": [
                    script,
                    {**script, "out_dir": str(directory / "stale")},
                    artifact,
                    finished,
                ],
                "different package": [
                    script,
                    {**artifact, "package_id": "other"},
                    finished,
                ],
                "backend authoring library": [
                    script,
                    {**artifact, "target": {"name": "dokime", "kind": ["lib"]}},
                    finished,
                ],
            }
            for scenario, selected in refused.items():
                with self.subTest(scenario=scenario):
                    messages.write_text("\n".join(map(json.dumps, selected)))
                    with self.assertRaises(ValueError):
                        locate(messages)

            messages.write_text("\n".join(map(json.dumps, events)))
            for contents in (
                "service.proto\n",
                "descriptors.bin\ndescriptors.bin\n",
                "descriptors.bin\n../service.proto\n",
                "descriptors.bin\n/missing.proto\n",
                "descriptors.bin\nmissing.proto\n",
                "descriptors.bin\nservice.rs\n",
                "descriptors.bin\n\n",
            ):
                with self.subTest(manifest=contents):
                    manifest.write_text(contents)
                    with self.assertRaises(ValueError):
                        locate(messages)

    def test_archive_transfer(self):
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            export = root / "selected/grpc/operations"
            export.mkdir(parents=True)
            assets = {
                "descriptors.bin": b"descriptor",
                "contracts.json": b"[]",
                "service.proto": b'syntax = "proto3";',
            }
            for name, data in assets.items():
                (export / name).write_bytes(data)
            (export / "manifest.txt").write_text("\n".join(assets) + "\n")
            package = "path+file:///selected/dokime-contract#0.1.0"
            events = [
                {
                    "reason": "build-script-executed",
                    "package_id": package,
                    "out_dir": str(export.parents[1]),
                },
                {
                    "reason": "compiler-artifact",
                    "package_id": package,
                    "target": {"name": "dokime-contract-export", "kind": ["bin"]},
                },
                {"reason": "build-finished", "success": True},
            ]
            messages = root / "cargo.jsonl"
            messages.write_text("\n".join(map(json.dumps, events)))
            revision = "a" * 40
            package_path = root / "contracts.tar"
            archive(messages, package_path, revision, root=root)
            digest = hashlib.sha256(package_path.read_bytes()).hexdigest()
            destination = root / "restored"
            restore(package_path, digest, revision, destination)
            self.assertEqual(restored(destination, revision), destination)
            transferred = root / "transferred.tar"
            archive(None, transferred, revision, root=root, export=destination)
            self.assertEqual(transferred.read_bytes(), package_path.read_bytes())
            for name, data in assets.items():
                self.assertEqual((destination / name).read_bytes(), data)

            # Readonly Consumers Refuse Different Inputs
            with self.assertRaises(ValueError):
                restore(package_path, "0" * 64, revision, destination)
            with self.assertRaises(ValueError):
                restore(package_path, digest, "b" * 40, destination)
            with self.assertRaises(ValueError):
                restored(destination, "b" * 40)
            (destination / "descriptors.bin").write_bytes(b"different")
            with self.assertRaises(ValueError):
                restored(destination, revision)
            restore(package_path, digest, revision, destination)
            self.assertEqual(restored(destination, revision), destination)
            unowned = root / "unowned"
            unowned.mkdir()
            (unowned / "important").write_bytes(b"preserve")
            with self.assertRaises(ValueError):
                restore(package_path, digest, revision, unowned)
            self.assertEqual((unowned / "important").read_bytes(), b"preserve")

            # Invalid Members Never Escape Staging
            for name in ("../escaped.proto", "./service.proto", "/absolute.proto"):
                hostile = root / "hostile.tar"
                with tarfile.open(hostile, "w") as output:
                    member = tarfile.TarInfo(name)
                    member.size = 1
                    output.addfile(member, io.BytesIO(b"x"))
                bad_digest = hashlib.sha256(hostile.read_bytes()).hexdigest()
                with self.assertRaises(ValueError):
                    restore(hostile, bad_digest, revision, destination)
                self.assertEqual(restored(destination, revision), destination)
            self.assertFalse((root / "escaped.proto").exists())

    def test_shared_exporter_projection_preserves_sdk_ownership(self):
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            export = root / "selected/grpc/operations"
            export.mkdir(parents=True)
            (export / "descriptors.bin").write_bytes(b"descriptor")
            (export / "contracts.json").write_text("[]")
            (export / "manifest.txt").write_text("descriptors.bin\ncontracts.json\n")
            package_id = "path+file:///selected/dokime-contract#0.1.0"
            messages = root / "build.jsonl"
            messages.write_text(
                "\n".join(
                    map(
                        json.dumps,
                        [
                            {
                                "reason": "build-script-executed",
                                "package_id": package_id,
                                "out_dir": str(export.parents[1]),
                            },
                            {
                                "reason": "compiler-artifact",
                                "package_id": package_id,
                                "target": {
                                    "name": "dokime-contract-export",
                                    "kind": ["bin"],
                                },
                            },
                            {"reason": "build-finished", "success": True},
                        ],
                    )
                )
            )
            sdk = root / "packages/dokime"
            destination = sdk / "src/dokime"
            client = destination / "client/client.py"
            client.parent.mkdir(parents=True)
            client.write_bytes(b"sdk behavior")
            (sdk / "pyproject.toml").write_text(
                '[project]\nname="dokime"\nversion="0.1.0"\ndescription="SDK"\nlicense="Apache-2.0"\n'
            )
            exports = b"# Product Client Exports\nfrom .client import Client\n"
            (destination / "__init__.py").write_bytes(b"old\n\n" + exports)
            (destination / "__init__.pyi").write_bytes(b"old\n\n" + exports)
            revision = b"generated-v1"
            hostile = False
            calls = []

            def exporter(arguments, **kwargs):
                calls.append(arguments)
                self.assertEqual(
                    arguments[:4], ["cargo", "harmos", "bindings", "python"]
                )
                self.assertEqual(
                    arguments[arguments.index("--export") + 1], str(export)
                )
                self.assertEqual(arguments[arguments.index("--module") + 1], "dokime")
                self.assertEqual(kwargs["cwd"], root)
                output = Path(arguments[arguments.index("--output") + 1]) / "src/dokime"
                for name in [
                    "__init__.py",
                    "__init__.pyi",
                    "operation.py",
                    "dynamic.py",
                    "py.typed",
                    "contracts.json",
                    "descriptors.bin",
                    "_wire/_pool.py",
                ]:
                    target = output / name
                    target.parent.mkdir(parents=True, exist_ok=True)
                    target.write_bytes(revision)
                if hostile:
                    target = output / "client/client.py"
                    target.parent.mkdir()
                    target.write_bytes(b"overwrite")

            with (
                patch("dokime_tools.bindings.prepare_harmos") as prepared,
                patch("dokime_tools.bindings.subprocess.run", side_effect=exporter),
                patch("dokime_tools.bindings._normalize"),
            ):
                generate(messages, root)
                self.assertEqual(client.read_bytes(), b"sdk behavior")
                generated = destination / "__init__.py"
                generate(messages, root, check=True)
                revision = b"generated-v2"
                with self.assertRaises(ValueError):
                    generate(messages, root, check=True)
                self.assertEqual(generated.read_bytes(), b"generated-v1\n\n" + exports)
                hostile = True
                with self.assertRaises(ValueError):
                    generate(messages, root)
                self.assertEqual(client.read_bytes(), b"sdk behavior")
                self.assertEqual(generated.read_bytes(), b"generated-v1\n\n" + exports)
            self.assertEqual(len(calls), 4)
            prepared.assert_called_with(root, allow_install=False, cargo_config=None)

            # Generated Formatting Preserves Opaque Assets And Product Exports
            source = root / "normalized"
            source.mkdir()
            generated = {
                Path("__init__.py"): b"import sys\nimport os\n\n" + exports,
                Path(
                    "__init__.pyi"
                ): b"import os\nfrom typing import Any\n\nvalue: Any\n",
                Path("dynamic.py"): b"VALUE={'a':1}\n",
                Path("descriptors.bin"): b"\x00\xffdescriptor",
                Path("_wire/_pool.py"): b"wire-owned bytes",
            }
            opaque = generated[Path("descriptors.bin")]
            wire = generated[Path("_wire/_pool.py")]
            _normalize(generated, source, root)
            normalized = generated.copy()
            self.assertIn(exports, generated[Path("__init__.py")])
            self.assertIn(b"import os", generated[Path("__init__.py")])
            self.assertNotIn(b"import os", generated[Path("__init__.pyi")])
            self.assertIn(b"from typing import Any", generated[Path("__init__.pyi")])
            self.assertEqual(generated[Path("descriptors.bin")], opaque)
            self.assertEqual(generated[Path("_wire/_pool.py")], wire)
            self.assertFalse((source / "descriptors.bin").exists())
            self.assertFalse((source / "_wire").exists())
            _normalize(generated, source, root)
            self.assertEqual(generated, normalized)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
