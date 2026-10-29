# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Hold A Hub Temporary Directory For Native Fixture Consumers"""

import sys

from . import temporary


def main() -> None:
    """Keep The Canonical Owner Alive Until Its Native Consumer Closes Stdin."""
    with temporary.directory() as directory:
        print(directory, flush=True)
        sys.stdin.buffer.read()


if __name__ == "__main__":
    main()
