# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

from __future__ import annotations

from dokime.core.suites import Suite

def suite_from_authored_suite(suite: type[object]) -> Suite: ...
