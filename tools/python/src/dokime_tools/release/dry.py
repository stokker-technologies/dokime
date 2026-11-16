# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Release Dry Operations."""

# === Imports === #

# Standard Library Imports
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.package.verification
import dokime_tools.screenshots.run

from .. import policy
from . import context, layout, staging, version

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def dry(
    directory: Path,
    settings: dict,
    *,
    profile: str | None = None,
    require_all: bool = False,
    source_revision: str | None = None,
    release_notes: str | None = None,
    targets: list[str] | None = None,
    channel: str = "stable",
) -> None:
    """Stage The Active Profile's Packages Like A Release And Run The Native Downloads From Them, Offline."""
    tag = context.release_tag(directory)
    version.run(directory, "check")
    # Release reviews package the release profile; every other run packages dev.
    profile = policy.profile("dev") if profile is None else profile
    root = context.root(directory)
    targets = list(settings.get("targets", [])) if targets is None else targets
    present = []
    for present_selected in targets:
        if not ((directory / "target/package" / profile / present_selected).is_dir()):
            continue
        present.append(present_selected)

    missing = sorted(set(targets) - set(present))
    if missing and (require_all or context.continuous_integration()):
        raise policy.ToolError(f"Rehearse Release Packages: {profile} packages are missing for release targets: {', '.join(missing)}")
    if missing:
        print(f"note: no local {profile} packages for {', '.join(missing)}; CI stages every release target")
    staging.stage(
        directory, settings, tag, present, profile=profile, source_revision=source_revision, release_notes=release_notes, channel=channel
    )
    files = layout.uploads(directory, staged=True)
    staging.checksums(root / context.SHARED / context.SUMS, {item["name"]: item for item in files})
    native = context.native()
    if native in present:
        # The staged bytes themselves run, from an isolated directory, exactly as a user's download would.
        dokime_tools.package.verification.verify(directory, profile, native, root / native)
    print(
        f"staged {len(files) - 2} download(s), {context.SUMS} and {context.INDEX} for {', '.join(present) or 'no target'} in {root.relative_to(directory)}"
    )


# === Private Functions === #

# === Entry Point === #
