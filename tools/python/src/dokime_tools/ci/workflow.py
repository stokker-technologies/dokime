# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Provider Operations And Release Ordering Behind Project Tasks."""

# === Imports === #

# Standard Library Imports
import argparse
import json
import os
import re
import runpy
import shlex
import subprocess
import tempfile
from pathlib import Path

# Third Party Imports
# Local Imports
from .. import policy
from ..process import spawn_flags
from . import transport

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def publication_guard() -> None:
    """Require Exact-Commit CI Before Tagging Or Publishing A Stable Release."""
    if not os.environ.get("RELEASE_TARGET_SHA"):
        return
    token = os.environ.get("GH_TOKEN")
    if os.environ.get("CI_READ_TOKEN"):
        os.environ["GH_TOKEN"] = os.environ["CI_READ_TOKEN"]
    try:
        runpy.run_path(str(Path(__file__).with_name("green.py")), run_name="__main__")
    finally:
        if token is None:
            os.environ.pop("GH_TOKEN", None)
        else:
            os.environ["GH_TOKEN"] = token
    from ..release import run

    if run.main(["identity"]):
        raise RuntimeError("trusted publishing identity failed")


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Run Provider Workflow Operations")
    parser.add_argument(
        "operation",
        choices=(
            "nightly-freshness",
            "nightly-report",
            "release-mode",
            "review",
            "required",
            "mirror",
            "release-prepare",
            "release-tag",
            "release-verify",
            "rehearse",
            "bindings",
        ),
    )
    parser.add_argument("--profile", choices=("dev", "release"), default="release")
    parser.add_argument("--target")
    parser.add_argument("--stage-only", action="store_true")
    parser.add_argument("--channel", choices=("stable", "nightly"), default="stable")
    options = parser.parse_args(arguments)
    transport.receive()
    operation = options.operation
    if operation == "nightly-freshness":
        from .review import pages, request

        if os.environ["GITHUB_REF"] != "refs/heads/dev":
            raise ValueError("nightly requires dev")
        build = True
        if os.environ["GITHUB_EVENT_NAME"] == "schedule":
            # A Missing Tag Is The First Nightly; Other API Failures Must Fail Closed.
            result = subprocess.run(
                ["gh", "api", "--include", f"repos/{os.environ['GITHUB_REPOSITORY']}/git/ref/tags/nightly"],
                check=False,
                capture_output=True,
                text=True,
                creationflags=spawn_flags(),
            )
            if result.returncode and not re.search(r"HTTP/\S+ 404", result.stdout) and "HTTP 404" not in result.stderr:
                raise RuntimeError("nightly tag lookup failed")
            if not result.returncode:
                published = request("git/ref/tags/nightly")
                build = published["object"]["sha"] != os.environ["GITHUB_SHA"]
        for status in ("in_progress", "queued"):
            runs = pages(f"actions/workflows/nightly.yml/runs?head_sha={os.environ['GITHUB_SHA']}&status={status}", "workflow_runs")
            if any(int(run["id"]) < int(os.environ["GITHUB_RUN_ID"]) for run in runs):
                build = False
                break
        with Path(os.environ["GITHUB_OUTPUT"]).open("a", encoding="utf-8") as output:
            output.write(f"build={str(build).lower()}\n")
    elif operation == "nightly-report":
        from ..release import nightly

        published = nightly.complete(policy.root())
        with Path(os.environ["GITHUB_OUTPUT"]).open("a", encoding="utf-8") as output:
            output.write(f"published={str(published).lower()}\n")
    elif operation == "release-mode":
        from .review import github_release_mode

        result = github_release_mode()
        with Path(os.environ["GITHUB_OUTPUT"]).open("a", encoding="utf-8") as output:
            output.write("".join(f"{key}={value}\n" for key, value in result.items()))
    elif operation == "review":
        runpy.run_path(str(Path(__file__).with_name("review.py")), run_name="__main__")
    elif operation == "required":
        results = json.loads(os.environ["RESULTS"])
        allowed_skips = set(json.loads(os.environ.get("OPTIONAL_JOBS", "[]")))
        failed = [
            name
            for name, job in results.items()
            if job["result"] != "success" and not (name in allowed_skips and job["result"] == "skipped")
        ]
        if failed:
            raise ValueError("required CI jobs failed or skipped: " + ", ".join(failed))
    elif operation == "mirror":
        if os.environ["GITHUB_REF_NAME"] not in ("dev", "main") or os.environ["GITHUB_REPOSITORY"] != "stokker-technologies/dokime":
            raise ValueError("mirror requires a published Dokime branch")
        sha = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True, creationflags=spawn_flags()).strip()
        if sha != os.environ["GITHUB_SHA"]:
            raise ValueError("mirror checkout differs from published revision")
        key = os.environ["GITLAB_MIRROR_SSH_KEY"]
        hosts = os.environ["GITLAB_MIRROR_KNOWN_HOSTS"]
        if not key.strip() or not hosts.strip():
            raise ValueError("mirror SSH credentials and known hosts are required")
        with tempfile.TemporaryDirectory(prefix="dokime-mirror-") as temporary:
            secret = Path(temporary) / "key"
            known = Path(temporary) / "hosts"
            secret.touch(mode=0o600)
            secret.write_text(key + "\n", encoding="utf-8")
            known.write_text(hosts + "\n", encoding="utf-8")
            ssh = f"ssh -i {shlex.quote(str(secret))} -o IdentitiesOnly=yes -o StrictHostKeyChecking=yes -o UserKnownHostsFile={shlex.quote(str(known))}"
            subprocess.run(
                [
                    "git",
                    "push",
                    "--no-force",
                    "--no-follow-tags",
                    "git@gitlab.com:stokker-technologies/open-source/dokime.git",
                    f"HEAD:refs/heads/{os.environ['GITHUB_REF_NAME']}",
                ],
                env=os.environ | {"GIT_SSH_COMMAND": ssh},
                check=True,
                creationflags=spawn_flags(),
            )
    elif operation == "bindings":
        from ..test.archive import install_wheels
        from . import tasks

        install_wheels(Path.cwd(), options.profile)
        return tasks.main(["check", "--bindings", "--profile", options.profile])
    else:
        from ..release import run

        if operation == "release-prepare":
            if run.main(["prepare"]):
                return 1
            from ..test import runners

            token = os.environ.get("GH_TOKEN", "")
            os.environ["GH_TOKEN"] = os.environ["RUNNER_GROUP_READ_TOKEN"]
            try:
                runners.main()
            finally:
                os.environ["GH_TOKEN"] = token
            return run.main(["review"])
        if operation == "release-tag":
            publication_guard()
            return run.main(["tag"])
        if operation == "release-verify":
            if options.channel == "nightly":
                from ..release import nightly

                nightly.verify(policy.root())
                return 0
            return run.main(["verify"])
        args = ["rehearse", "--profile", options.profile]
        if options.target:
            args += ["--target", options.target]
        if options.stage_only:
            args += ["--stage-only"]
        return run.main(args)
    transport.send(options.profile)
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
