# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own the Complete Check and Review Test Matrices."""

# === Imports === #

# Standard Library Imports
from typing import Any

# isort: split

# Third Party Imports

# isort: split

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

SDKS = {"rust": "Rust", "python": "Python", "csharp": "C#", "cpp": "C++"}
# CI Validates and Releases Ship Only These SDKs; C# and C++ Stay Buildable by Hand Until Their CI Is Repaired.
SDK_LANGUAGES = ("rust", "python")

CHECKS: tuple[dict[str, Any], ...] = (
    {
        "platform": "unix",
        "runner": "ubuntu-latest",
        "minutes": 120,
        "work": "Repository",
        "command": "check -- --repository",
        "bootstrap": "[]",
        "inputs": "",
        "language": "repository",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-latest",
        "minutes": 30,
        "work": "Host / Supply Chain",
        "command": "check -- --supply",
        "bootstrap": '["mise run ci:prerequisites -- --gui-packages"]',
        "inputs": "",
        "language": "rust",
    },
    {
        "platform": "windows",
        "runner": "windows-latest",
        "minutes": 30,
        "work": "Host / Supply Chain",
        "command": "check -- --supply",
        "bootstrap": "[]",
        "inputs": "",
        "language": "rust",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-latest",
        "minutes": 120,
        "work": "SDK / Python",
        "command": "check -- --language python",
        "bootstrap": "[]",
        "inputs": "",
        "language": "python",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-latest",
        "minutes": 120,
        "work": "Desktop",
        "command": "check -- --language typescript",
        "bootstrap": "[]",
        "inputs": "",
        "language": "typescript",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-latest",
        "minutes": 120,
        "work": "Host",
        "command": "check -- --language rust",
        "bootstrap": '["mise run ci:prerequisites -- --gui-packages"]',
        "inputs": "",
        "language": "rust",
    },
    {
        "platform": "windows",
        "runner": "windows-latest",
        "minutes": 120,
        "work": "Host",
        "command": "check -- --language rust",
        "bootstrap": "[]",
        "inputs": "",
        "language": "rust",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-22.04",
        "minutes": 120,
        "work": "SDK / C#",
        "command": "ci:sdk-check -- --language csharp",
        "bootstrap": "[]",
        "inputs": "",
        "language": "csharp",
    },
    {
        "platform": "windows",
        "runner": "windows-latest",
        "minutes": 120,
        "work": "SDK / C#",
        "command": "ci:sdk-check -- --language csharp",
        "bootstrap": "[]",
        "inputs": "",
        "language": "csharp",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-22.04",
        "minutes": 120,
        "work": "SDK / C++",
        "command": "ci:sdk-check -- --language cpp",
        "bootstrap": "[]",
        "inputs": "",
        "language": "cpp",
    },
    {
        "platform": "windows",
        "runner": "windows-latest",
        "minutes": 120,
        "work": "SDK / C++",
        "command": "ci:sdk-check -- --language cpp",
        "bootstrap": "[]",
        "inputs": "",
        "language": "cpp",
    },
)

TESTS: tuple[dict[str, Any], ...] = (
    {
        "platform": "unix",
        "runner": "ubuntu-22.04",
        "minutes": 120,
        "work": "SDK / Python / Wheel",
        "command": "test -- --language python --wheel",
        "bootstrap": '["mise run ci:prerequisites -- --credential-store"]',
        "archives": '["target/ci/build-product-python-{profile}-any.tar.zst","target/ci/build-tests-{profile}-{platform}.tar.zst"]',
        "language": "python",
    },
    {
        "platform": "windows",
        "runner": "windows-latest",
        "minutes": 120,
        "work": "SDK / Python / Wheel",
        "command": "test -- --language python --wheel",
        "bootstrap": "[]",
        "archives": '["target/ci/build-product-python-{profile}-any.tar.zst","target/ci/build-tests-{profile}-{platform}.tar.zst"]',
        "language": "python",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-22.04",
        "minutes": 120,
        "work": "Desktop / Execution",
        "command": "test -- --language typescript --layer system --select native-desktop",
        "bootstrap": '["mise run ci:prerequisites -- --gui-runtime --desktop"]',
        "archives": '["target/ci/build-cli-{profile}-{platform}.tar.zst","target/ci/build-clients-{profile}-{platform}.tar.zst","target/ci/build-tests-{profile}-{platform}.tar.zst"]',
        "language": "typescript",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-22.04",
        "minutes": 120,
        "work": "Desktop / Connections",
        "command": "test -- --language typescript --layer system --select two-gui-remote",
        "bootstrap": '["mise run ci:prerequisites -- --gui-runtime --desktop"]',
        "archives": '["target/ci/build-cli-{profile}-{platform}.tar.zst","target/ci/build-clients-{profile}-{platform}.tar.zst","target/ci/build-tests-{profile}-{platform}.tar.zst"]',
        "language": "typescript",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-22.04",
        "minutes": 120,
        "work": "Host / Acceptance",
        "command": "test -- --language rust --layer acceptance",
        "bootstrap": '["mise run ci:prerequisites -- --gui-runtime"]',
        "archives": '["target/ci/build-tests-{profile}-{platform}.tar.zst","target/ci/build-clients-{profile}-{platform}.tar.zst"]',
        "language": "rust",
    },
    {
        "platform": "windows",
        "runner": "windows-latest",
        "minutes": 120,
        "work": "Desktop / Execution",
        "command": "test -- --language typescript --layer system --select native-desktop",
        "bootstrap": '["mise run ci:prerequisites -- --desktop"]',
        "archives": '["target/ci/build-cli-{profile}-{platform}.tar.zst","target/ci/build-clients-{profile}-{platform}.tar.zst","target/ci/build-tests-{profile}-{platform}.tar.zst"]',
        "language": "typescript",
    },
    {
        "platform": "windows",
        "runner": "windows-latest",
        "minutes": 120,
        "work": "Desktop / Connections",
        "command": "test -- --language typescript --layer system --select two-gui-remote",
        "bootstrap": '["mise run ci:prerequisites -- --desktop"]',
        "archives": '["target/ci/build-cli-{profile}-{platform}.tar.zst","target/ci/build-clients-{profile}-{platform}.tar.zst","target/ci/build-tests-{profile}-{platform}.tar.zst"]',
        "language": "typescript",
    },
    {
        "platform": "windows",
        "runner": "windows-latest",
        "minutes": 120,
        "work": "Host / Acceptance",
        "command": "test -- --language rust --layer acceptance",
        "bootstrap": "[]",
        "archives": '["target/ci/build-tests-{profile}-{platform}.tar.zst","target/ci/build-clients-{profile}-{platform}.tar.zst"]',
        "language": "rust",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-latest",
        "minutes": 120,
        "work": "Desktop / Unit",
        "command": "test -- --language typescript --layer unit",
        "bootstrap": "[]",
        "archives": '["target/ci/build-clients-{profile}-unix.tar.zst"]',
        "language": "typescript",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-22.04",
        "minutes": 120,
        "work": "SDK / Python / Unit",
        "command": "test -- --language python --layer unit",
        "bootstrap": "[]",
        "archives": '["target/ci/build-product-python-{profile}-any.tar.zst"]',
        "language": "python",
    },
    {
        "platform": "windows",
        "runner": "windows-latest",
        "minutes": 120,
        "work": "SDK / Python / Unit",
        "command": "test -- --language python --layer unit",
        "bootstrap": "[]",
        "archives": '["target/ci/build-product-python-{profile}-any.tar.zst"]',
        "language": "python",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-22.04",
        "minutes": 120,
        "work": "SDK / Python / Integration",
        "command": "test -- --language python --layer integration",
        "bootstrap": '["mise run ci:prerequisites -- --credential-store"]',
        "archives": '["target/ci/build-product-python-{profile}-any.tar.zst","target/ci/build-tests-{profile}-{platform}.tar.zst"]',
        "language": "python",
    },
    {
        "platform": "windows",
        "runner": "windows-latest",
        "minutes": 120,
        "work": "SDK / Python / Integration",
        "command": "test -- --language python --layer integration",
        "bootstrap": "[]",
        "archives": '["target/ci/build-product-python-{profile}-any.tar.zst","target/ci/build-tests-{profile}-{platform}.tar.zst"]',
        "language": "python",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-latest",
        "minutes": 120,
        "work": "Desktop / Integration",
        "command": "test -- --language typescript --layer integration",
        "bootstrap": "[]",
        "archives": '["target/ci/build-clients-{profile}-unix.tar.zst"]',
        "language": "typescript",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-latest",
        "minutes": 120,
        "work": "Host / Integration",
        "command": "test -- --language rust --layer integration",
        "bootstrap": '["mise run ci:prerequisites -- --gui-runtime"]',
        "archives": '["target/ci/build-tests-{profile}-{platform}.tar.zst"]',
        "language": "rust",
    },
    {
        "platform": "windows",
        "runner": "windows-latest",
        "minutes": 120,
        "work": "Host / Integration",
        "command": "test -- --language rust --layer integration",
        "bootstrap": "[]",
        "archives": '["target/ci/build-tests-{profile}-{platform}.tar.zst"]',
        "language": "rust",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-latest",
        "minutes": 120,
        "work": "Host / System",
        "command": 'test -- --language rust --layer system --select "not package(dokime-runtime)"',
        "bootstrap": '["mise run ci:prerequisites -- --gui-runtime"]',
        "archives": '["target/ci/build-product-python-{profile}-any.tar.zst","target/ci/build-cli-{profile}-{platform}.tar.zst","target/ci/build-tests-{profile}-{platform}.tar.zst"]',
        "language": "rust",
    },
    {
        "platform": "windows",
        "runner": "windows-latest",
        "minutes": 120,
        "work": "Host / System",
        "command": 'test -- --language rust --layer system --select "not package(dokime-runtime)"',
        "bootstrap": "[]",
        "archives": '["target/ci/build-product-python-{profile}-any.tar.zst","target/ci/build-cli-{profile}-{platform}.tar.zst","target/ci/build-tests-{profile}-{platform}.tar.zst"]',
        "language": "rust",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-latest",
        "minutes": 120,
        "work": "Host / System / Execution",
        "nightly": False,
        "command": 'test -- --artifact tests --package dokime-runtime --target system --filterset "not '
        '(test(csharp_plugin_matches_rust_thermal_evidence) or test(cpp_plugin_matches_rust_thermal_evidence))"',
        "bootstrap": '["mise run ci:prerequisites -- --gui-runtime"]',
        "archives": '["target/ci/build-tests-{profile}-{platform}.tar.zst","target/ci/build-product-python-{profile}-any.tar.zst"]',
        "language": "rust",
    },
    {
        "platform": "windows",
        "runner": "windows-latest",
        "minutes": 120,
        "work": "Host / System / Execution",
        "nightly": False,
        "command": 'test -- --artifact tests --package dokime-runtime --target system --filterset "not '
        '(test(csharp_plugin_matches_rust_thermal_evidence) or test(cpp_plugin_matches_rust_thermal_evidence))"',
        "bootstrap": "[]",
        "archives": '["target/ci/build-tests-{profile}-{platform}.tar.zst","target/ci/build-product-python-{profile}-any.tar.zst"]',
        "language": "rust",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-latest",
        "minutes": 120,
        "work": "Host / Unit",
        "command": "test -- --language rust --layer unit",
        "bootstrap": '["mise run ci:prerequisites -- --gui-runtime"]',
        "archives": '["target/ci/build-tests-{profile}-{platform}.tar.zst"]',
        "language": "rust",
    },
    {
        "platform": "windows",
        "runner": "windows-latest",
        "minutes": 120,
        "work": "Host / Unit",
        "command": "test -- --language rust --layer unit",
        "bootstrap": "[]",
        "archives": '["target/ci/build-tests-{profile}-{platform}.tar.zst"]',
        "language": "rust",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-22.04",
        "minutes": 120,
        "work": "SDK / C#",
        "command": "ci:sdk-test -- --language csharp",
        "bootstrap": "[]",
        "archives": "[]",
        "language": "csharp",
    },
    {
        "platform": "windows",
        "runner": "windows-latest",
        "minutes": 120,
        "work": "SDK / C#",
        "command": "ci:sdk-test -- --language csharp",
        "bootstrap": "[]",
        "archives": "[]",
        "language": "csharp",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-22.04",
        "minutes": 120,
        "work": "C++ / SDK",
        "command": "ci:sdk-test -- --language cpp",
        "bootstrap": "[]",
        "archives": "[]",
        "language": "cpp",
    },
    {
        "platform": "windows",
        "runner": "windows-latest",
        "minutes": 120,
        "work": "C++ / SDK",
        "command": "ci:sdk-test -- --language cpp",
        "bootstrap": "[]",
        "archives": "[]",
        "language": "cpp",
    },
    {
        "platform": "unix",
        "runner": "ubuntu-22.04",
        "minutes": 120,
        "work": "SDK / Python / System",
        "command": "test -- --language python --layer system",
        "bootstrap": '["mise run ci:prerequisites -- --credential-store"]',
        "archives": '["target/ci/build-product-python-{profile}-any.tar.zst","target/ci/build-cli-{profile}-{platform}.tar.zst","target/ci/build-tests-{profile}-{platform}.tar.zst"]',
        "language": "python",
    },
    {
        "platform": "windows",
        "runner": "windows-latest",
        "minutes": 120,
        "work": "SDK / Python / System",
        "command": "test -- --language python --layer system",
        "bootstrap": "[]",
        "archives": '["target/ci/build-product-python-{profile}-any.tar.zst","target/ci/build-cli-{profile}-{platform}.tar.zst","target/ci/build-tests-{profile}-{platform}.tar.zst"]',
        "language": "python",
    },
)

# === Types === #

# === Classes === #

# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
