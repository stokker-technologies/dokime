# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"Parse Release Arguments and Invoke Their Owning Operations."

# === Imports === #

# Standard Library Imports
import argparse
import json
import os
import sys
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy
from . import (
    context,
    dry,
    finalize,
    identity,
    prepare,
    publish,
    registry,
    rehearse,
    review,
    staging,
    tag,
    verify,
)

# Type Checking Imports

# === Public Exports === #

# === Constants === #

CHANNEL_VERBS = ("rehearse", "crates-dry")
VERBS = [
    "prepare",
    "review",
    "dry",
    "crates-dry",
    "rehearse",
    "identity",
    "tag",
    "crates",
    "publish",
    "python",
    "python-dry",
    "finalize",
    "verify",
    "guard",
    "assets",
    "stage",
]

# === Types === #

# === Classes === #

# === Public Functions === #


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("verb", choices=VERBS)
    parser.add_argument("--target", help="Rehearse native, one declared release target, or all targets; hosted runs require all.")
    parser.add_argument("--profile", choices=("dev", "release"), help="Local rehearsal package profile; defaults to release.")
    parser.add_argument("--stage-only", action="store_true", help="Local rehearsal staging without registry dry-runs.")
    parser.add_argument("--channel", choices=("stable", "nightly"))
    options = parser.parse_args(arguments)
    try:
        channel = options.channel or (os.environ.get("RELEASE_CHANNEL") if options.verb in CHANNEL_VERBS else None) or "stable"
        if channel not in ("stable", "nightly"):
            raise policy.ToolError("Select Release Channel: Set RELEASE_CHANNEL to stable or nightly")
        run(
            policy.root(),
            options.verb,
            target=options.target,
            profile=options.profile,
            stage_only=options.stage_only,
            channel=channel,
        )
    except (policy.ToolError, OSError, ValueError, KeyError, TypeError) as error:
        print(f"release: {error}", file=sys.stderr)
        return 1
    return 0


def run(
    directory: Path, verb: str, *, target: str | None = None, profile: str | None = None, stage_only: bool = False, channel: str = "stable"
) -> None:
    if verb != "rehearse" and (target is not None or profile is not None or stage_only):
        raise policy.ToolError("Select Rehearsal: --target, --profile and --stage-only Are Supported Only by Rehearsal")
    if verb not in CHANNEL_VERBS and channel != "stable":
        raise policy.ToolError("Select Channel: --channel Is Supported Only by Rehearsal and the Crate Dry Run")
    settings = policy.policy(directory, "release")
    if verb == "prepare":
        prepare.prepare(directory, settings)
        return
    if verb == "review":
        review.review(directory, settings)
        return
    if verb == "dry":
        dry.dry(directory, settings)
        return
    if verb == "rehearse":
        rehearse.rehearse(directory, settings, target=target, profile=profile or "release", stage_only=stage_only, channel=channel)
        return
    if verb == "identity":
        identity.identity(directory, settings)
        return
    if verb == "crates-dry":
        registry.crates.rehearse(directory, skip_published=False, channel=channel)
        return
    if verb == "tag":
        tag.tag(directory, settings)
        return
    if verb == "crates":
        registry.crates.publish(directory)
        return
    if verb == "publish":
        publish.publish(directory, settings)
        return
    if verb == "python":
        registry.python.publish(directory, settings, dry_run=False)
        return
    if verb == "python-dry":
        registry.python.publish(directory, settings, dry_run=True)
        return
    if verb == "finalize":
        finalize.finalize(directory, settings)
        return
    if verb == "verify":
        verify.verify(directory, settings)
        return
    if verb == "guard":
        registry.provider.require_tag(directory, context.release_tag(directory))
        return
    if verb == "assets":
        print(
            json.dumps(
                staging.assets(directory, settings, context.native_target(settings), context.release_tag(directory), signed=True), indent=2
            )
        )
        return
    if verb == "stage":
        staging.stage(directory, settings, context.release_tag(directory), settings["targets"], signed=True)
        return
    raise policy.ToolError(f"Select Release Operation: Choose One of {', '.join(VERBS)}")


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
