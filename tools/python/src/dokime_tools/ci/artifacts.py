# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Reuse Exact Native GitHub Actions Validation Artifacts"""

# === Imports === #

# Standard Library Imports
import argparse
import hashlib
import json
import os
import re
import shutil
import stat
import tempfile
from pathlib import Path, PurePosixPath
from urllib.error import HTTPError
from urllib.parse import urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener
from zipfile import ZipFile

# Third Party Imports
# Local Imports
from dokime_tools.check.release import PLATFORMS, digest, inputs, verify_platform
from dokime_tools.ci.policy import current_event, required_platforms

# Type Checking Imports

# === Public Exports === #

# === Constants === #
CATALOG = "dist/validation/catalog.json"
REPOSITORY = "stokker-technologies/dokime"
BUDGET = 1024 * 1024 * 1024

# === Types === #

# === Classes === #


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


# === Public Functions === #


def job_id(
    task: str, platform: str | None = None, *, completed: bool = False
) -> str | None:
    if os.environ.get("GITHUB_ACTIONS") != "true":
        return None
    run = _job(os.environ["GITHUB_RUN_ID"])
    label = task.replace(":", " ").replace("-", " ").title()
    name = f"{label} ({platform})" if platform else label
    jobs = _request(f"actions/runs/{run}/jobs?per_page=100")
    if jobs.get("total_count", 0) > 100:
        raise ValueError("native job inventory exceeds its budget")
    matches = [
        job
        for job in jobs["jobs"]
        if (job.get("name") == name or job.get("name", "").endswith(" / " + name))
    ]
    if len(matches) != 1:
        raise ValueError(f"native job identity is ambiguous: {name}")
    if completed and (
        matches[0].get("status") != "completed"
        or matches[0].get("conclusion") != "success"
    ):
        raise ValueError(f"native job did not pass: {name}")
    return _job(str(matches[0]["id"]))


def restore(root: Path, platform: str) -> None:
    archive = root / "dist/downloads" / f"dokime-{platform}.zip"
    destination = root / "dist/build" / platform
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=destination.parent) as temporary:
        staging = Path(temporary)
        _extract(archive, staging, platform)
        package = staging / "dist/build" / platform
        verify_platform(
            package, platform, root, require_validation=False, check_tools=False
        )
        if destination.exists():
            shutil.rmtree(destination)
        package.rename(destination)
    report = root / "crates/dokime-app/dist-report/report.html"
    report.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=report.parent) as temporary:
        installed = Path(temporary) / report.name
        shutil.copyfile(destination / "reports/report.html", installed)
        installed.replace(report)


def catalog(root: Path) -> None:
    identity = inputs(root)
    if not identity["source_tree"]:
        raise ValueError("validation catalogs require clean committed source")
    run = _job(os.environ["GITHUB_RUN_ID"])
    available = _inventory(run)
    selected = required_platforms(current_event())
    platforms = {}
    for platform in selected:
        restore(root, platform)
        package = root / "dist/build" / platform
        manifest = verify_platform(
            package, platform, root, require_validation=True, check_tools=False
        )
        validation = manifest["validation"]
        if manifest["pipeline_id"] != run or validation["pipeline_id"] != run:
            raise ValueError("native jobs belong to different validation runs")
        original = available[f"validation-{platform}-{validation['test_job_id']}"]
        _valid_artifact(original)
        archive = root / "dist/downloads" / f"dokime-{platform}.zip"
        platforms[platform] = {
            "artifact_id": _job(str(original["id"])),
            "artifact_sha256": _checksum(original),
            "build_job_id": _job(manifest["build_job_id"]),
            "test_job_id": _job(validation["test_job_id"]),
            "manifest_sha256": digest(package / "manifest.json"),
            "archive": {
                "path": archive.relative_to(root).as_posix(),
                "sha256": digest(archive),
                "bytes": archive.stat().st_size,
            },
        }
    jobs = [
        entry[key]
        for entry in platforms.values()
        for key in ("build_job_id", "test_job_id")
    ]
    if len(set(jobs)) != len(jobs):
        raise ValueError("native platform jobs must be distinct")
    native = _request(f"actions/runs/{run}")
    result = {
        "version": 1,
        "catalog_job_id": job_id("ci:artifacts:catalog"),
        "pipeline_id": run,
        "run_attempt": _job(os.environ["GITHUB_RUN_ATTEMPT"]),
        "head_revision": native["head_sha"],
        "inputs": identity,
        "platforms": platforms,
        "validation_policy": {
            "required_platforms": list(selected),
        },
    }
    path = root / CATALOG
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    destination = os.environ.get("GITHUB_OUTPUT")
    if destination:
        with Path(destination).open("a", encoding="utf-8") as output:
            output.write(
                f"artifact_name=validation-catalog-{result['catalog_job_id']}\n"
            )


def reuse(root: Path) -> None:
    if os.environ.get("GITHUB_EVENT_NAME") == "pull_request":
        raise ValueError("pull requests require fresh native build and validation")
    identity = inputs(root)
    if not identity.get("source_tree"):
        raise ValueError("immutable promotion requires clean committed source")
    reference = _job(os.environ.get("VALIDATED_RUN_ARTIFACT_ID"))
    catalog_artifact = _request(f"actions/artifacts/{reference}")
    _valid_artifact(catalog_artifact)
    if str(catalog_artifact["id"]) != reference or not re.fullmatch(
        r"validation-catalog-[1-9][0-9]*", catalog_artifact["name"]
    ):
        raise ValueError("original catalog artifact identity mismatch")
    cache = root / ".cache"
    cache.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="validation-", dir=cache) as temporary:
        staging = Path(temporary)
        archive = staging / "catalog.zip"
        _download(catalog_artifact, archive)
        with ZipFile(archive) as contents:
            if (
                contents.namelist() != ["catalog.json"]
                or contents.getinfo("catalog.json").file_size > 4 * 1024 * 1024
            ):
                raise ValueError("invalid native validation catalog artifact")
            original = contents.read("catalog.json")
        manifest = json.loads(original)
        run = _job(manifest.get("pipeline_id"))
        attempt = _job(manifest.get("run_attempt"))
        native = _request(f"actions/runs/{run}/attempts/{attempt}")
        if native.get("status") != "completed" or native.get("conclusion") != "success":
            raise ValueError("original validation run is incomplete or failed")
        if (
            manifest.get("version") != 1
            or str(catalog_artifact["workflow_run"]["id"]) != run
            or catalog_artifact["name"]
            != f"validation-catalog-{manifest.get('catalog_job_id')}"
            or manifest.get("head_revision") != native.get("head_sha")
            or manifest.get("inputs") != identity
            or set(manifest.get("platforms", {})) != set(PLATFORMS)
        ):
            raise ValueError(
                "original validation source, inputs or platform completeness mismatch"
            )
        for platform in PLATFORMS:
            entry = manifest["platforms"][platform]
            artifact = _request(f"actions/artifacts/{_job(entry['artifact_id'])}")
            _valid_artifact(artifact)
            if (
                str(artifact["workflow_run"]["id"]) != run
                or artifact["name"] != f"validation-{platform}-{entry['test_job_id']}"
            ):
                raise ValueError("original native artifact ownership mismatch")
            if (
                _job(str(artifact["id"])) != entry["artifact_id"]
                or _checksum(artifact) != entry["artifact_sha256"]
            ):
                raise ValueError("original native artifact identity mismatch")
            archive = staging / f"{platform}.zip"
            _download(artifact, archive)
            download = staging / "dist/downloads" / f"dokime-{platform}.zip"
            download.parent.mkdir(parents=True, exist_ok=True)
            with ZipFile(archive) as contents:
                if (
                    contents.namelist() != [download.name]
                    or contents.getinfo(download.name).file_size > BUDGET
                ):
                    raise ValueError("original native download archive is missing")
                with (
                    contents.open(download.name) as source,
                    download.open("wb") as output,
                ):
                    shutil.copyfileobj(source, output)
            expected = entry["archive"]
            if (
                expected["path"] != download.relative_to(staging).as_posix()
                or download.stat().st_size != expected["bytes"]
                or digest(download) != expected["sha256"]
            ):
                raise ValueError(f"original {platform} download archive mismatch")
            _extract(download, staging, platform)
            package = staging / "dist/build" / platform
            record = verify_platform(
                package, platform, root, require_validation=True, check_tools=False
            )
            if (
                digest(package / "manifest.json") != entry["manifest_sha256"]
                or record["build_job_id"] != entry["build_job_id"]
                or record["validation"]["test_job_id"] != entry["test_job_id"]
                or record["pipeline_id"] != run
                or record["validation"]["pipeline_id"] != run
            ):
                raise ValueError(f"original {platform} job provenance mismatch")
        for platform in PLATFORMS:
            package = root / "dist/build" / platform
            package.parent.mkdir(parents=True, exist_ok=True)
            if package.exists():
                shutil.rmtree(package)
            (staging / "dist/build" / platform).rename(package)
            download = root / "dist/downloads" / f"dokime-{platform}.zip"
            download.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(staging / "dist/downloads" / download.name, download)
        destination = root / CATALOG
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(original)


# === Private Functions === #


def _job(value: object) -> str:
    if not isinstance(value, str) or not re.fullmatch(r"[1-9][0-9]*", value):
        raise ValueError("one immutable validation run reference is required")
    return value


def _valid_artifact(artifact: dict) -> None:
    _job(str(artifact["id"]))
    if artifact.get("expired") is not False:
        raise ValueError("original native artifact is expired")
    _checksum(artifact)


def _checksum(artifact: dict) -> str:
    value = artifact.get("digest", "")
    if not isinstance(value, str) or not re.fullmatch(r"sha256:[a-f0-9]{64}", value):
        raise ValueError("native artifact checksum is missing")
    return value[7:]


def _inventory(run: str) -> dict[str, dict]:
    response = _request(f"actions/runs/{run}/artifacts?per_page=100")
    if response.get("total_count", 0) > 100:
        raise ValueError("native artifact inventory exceeds its budget")
    result = {}
    for artifact in response["artifacts"]:
        name = artifact["name"]
        if not name.startswith("validation-"):
            continue
        if (
            name in result
            or artifact.get("expired") is not False
            or str(artifact["workflow_run"]["id"]) != run
        ):
            raise ValueError("native validation artifact is ambiguous or expired")
        _checksum(artifact)
        result[name] = artifact
    return result


def _headers() -> dict[str, str]:
    if (
        os.environ.get("GITHUB_REPOSITORY") != REPOSITORY
        or os.environ.get("GITHUB_API_URL", "https://api.github.com")
        != "https://api.github.com"
    ):
        raise ValueError("validation retrieval requires the configured GitHub origin")
    return {
        "Authorization": f"Bearer {os.environ['GITHUB_TOKEN']}",
        "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
    }


def _request(endpoint: str) -> dict:
    request = Request(
        f"https://api.github.com/repos/{REPOSITORY}/{endpoint}", headers=_headers()
    )
    with build_opener(NoRedirect()).open(request, timeout=60) as response:
        contents = response.read(4 * 1024 * 1024 + 1)
    if len(contents) > 4 * 1024 * 1024:
        raise ValueError("native metadata exceeds its budget")
    value = json.loads(contents)
    if not isinstance(value, dict):
        raise ValueError("invalid native metadata")
    return value


def _download(artifact: dict, destination: Path) -> None:
    identity = _job(str(artifact["id"]))
    url = f"https://api.github.com/repos/{REPOSITORY}/actions/artifacts/{identity}/zip"
    headers = _headers()
    opener = build_opener(NoRedirect())
    checksum = hashlib.sha256()
    for _ in range(4):
        try:
            with (
                opener.open(Request(url, headers=headers), timeout=60) as response,
                destination.open("wb") as output,
            ):
                total = 0
                while chunk := response.read(1024 * 1024):
                    total += len(chunk)
                    if total > BUDGET:
                        raise ValueError(
                            "native artifact archive exceeds its size budget"
                        )
                    checksum.update(chunk)
                    output.write(chunk)
            if checksum.hexdigest() != _checksum(artifact):
                raise ValueError("native artifact checksum mismatch")
            return
        except HTTPError as error:
            if error.code not in (301, 302, 303, 307, 308):
                raise ValueError(
                    f"original artifacts unavailable: artifact {identity}, HTTP {error.code}"
                ) from error
            url = error.headers.get("Location", "")
            redirect = urlsplit(url)
            if (
                redirect.scheme != "https"
                or not redirect.netloc
                or redirect.username
                or redirect.password
            ):
                raise ValueError("invalid artifact storage redirect") from error
            headers = {}
    raise ValueError("too many artifact storage redirects")


def _extract(archive: Path, destination: Path, platform: str) -> None:
    target_root = destination / "dist/build" / platform
    selected = []
    size = 0
    with ZipFile(archive) as contents:
        names = set()
        for entry in contents.infolist():
            name = entry.filename
            path = PurePosixPath(name)
            if (
                path.is_absolute()
                or path.as_posix() != name.rstrip("/")
                or ".." in path.parts
                or "\\" in name
                or ":" in name
                or stat.S_ISLNK(entry.external_attr >> 16)
            ):
                raise ValueError("unsafe native artifact archive entry")
            if name in names:
                raise ValueError("duplicate native artifact archive entry")
            names.add(name)
            size += entry.file_size
            if size > BUDGET:
                raise ValueError("native extracted artifacts exceed their size budget")
            selected.append(entry)
        if "manifest.json" not in names:
            raise ValueError(
                "original native manifest is missing; rebuilding is forbidden"
            )
        for entry in selected:
            target = target_root / entry.filename
            if entry.is_dir():
                target.mkdir(parents=True, exist_ok=True)
                continue
            target.parent.mkdir(parents=True, exist_ok=True)
            with contents.open(entry) as source, target.open("wb") as output:
                shutil.copyfileobj(source, output)
            mode = (entry.external_attr >> 16) & 0o777
            if mode:
                target.chmod(mode)


# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("catalog", "reuse", "restore", "assemble"))
    parser.add_argument("--platform", choices=PLATFORMS)
    arguments = parser.parse_args()
    if arguments.command == "restore":
        if not arguments.platform:
            parser.error("restore requires --platform")
        restore(Path.cwd(), arguments.platform)
    elif arguments.command == "assemble":
        for platform in PLATFORMS:
            restore(Path.cwd(), platform)
    else:
        {"catalog": catalog, "reuse": reuse}[arguments.command](Path.cwd())
