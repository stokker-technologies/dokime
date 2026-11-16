# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Own Provider Operations and Release Ordering Behind Project Tasks."

# === Imports === #

# Standard Library Imports
import argparse
import os

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy
from ..release import guard
from . import bindings, mirror, nightly_freshness, nightly_report, release_mode, required, review, transport

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


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
        nightly_freshness.run()
    elif operation == "nightly-report":
        nightly_report.run()
    elif operation == "release-mode":
        release_mode.run()
    elif operation == "review":
        review.main()
    elif operation == "required":
        required.run()
    elif operation == "mirror":
        mirror.run()
    elif operation == "bindings":
        return bindings.run(options)
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
            guard.check()
            return run.main(["tag"])
        if operation == "release-verify":
            if options.channel == "nightly":
                from ..release import nightly

                nightly.verify(policy.root())
                return 0
            return run.main(["verify"])
        args = ["rehearse", "--profile", options.profile, "--channel", options.channel]
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
