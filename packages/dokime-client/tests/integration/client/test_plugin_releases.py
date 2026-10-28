# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Generate Release Metadata From Canonically Inspected Bundles"""

import hashlib
import json
from pathlib import Path
from types import SimpleNamespace

import pytest
from dokime_tools.package import plugins


def test_release_feed_uses_inspected_identity_targets_and_complete_digest(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    bundle = tmp_path / "instruments.dokime"
    bundle.write_bytes(b"complete inspected bundle")
    manifest = {"identity": {"name": "instruments", "version": "1.2.3"}, "files": [{"executable": {"target": "x86_64-pc-windows-msvc"}}]}
    calls = []

    def inspect(command: list[str], **kwargs: object) -> SimpleNamespace:
        calls.append(command)
        assert kwargs["check"] is True
        return SimpleNamespace(stdout=json.dumps({"manifest": manifest}))

    monkeypatch.setattr(plugins.subprocess, "run", inspect)
    tool = tmp_path / "cargo-dokime"
    assets = [(bundle, "https://example.test/instruments.dokime")]
    feed = plugins.release_feed(assets, tool=tool, requires_dokime="=0.1.0")

    assert calls == [[str(tool), "dokime", "sidecar", "inspect", str(bundle)]]
    assert (feed["format_version"], feed["package"], feed["version"]) == (1, "instruments", "1.2.3")
    assert feed["assets"] == [
        {"target": "x86_64-pc-windows-msvc", "url": assets[0][1], "sha256": hashlib.sha256(bundle.read_bytes()).hexdigest()}
    ]
    with pytest.raises(ValueError, match="distinct executable targets"):
        plugins.release_feed(assets * 2, tool=tool, requires_dokime="=0.1.0")
    with pytest.raises(ValueError, match="credential-free HTTPS"):
        plugins.release_feed([(bundle, "https://token@example.test/plugin")], tool=tool, requires_dokime="=0.1.0")
