/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Native Suite Reads
 */

/* === Imports === */

import { required } from "@dokime/plots/lib/rpc/canonical";

import type { Suite } from "@generated/host/model";

import type { Wire } from "@dokime/plots/lib/rpc/canonical";

import type { SessionIdentity } from "@/lib/rpc/transport";

import type { TestSuiteEntry } from "@dokime/plots/lib/types";

import * as client from "./client";

import { mapSuite } from "@dokime/document/lib/host/suites";

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export async function fetchNativeSuites(session: SessionIdentity): Promise<TestSuiteEntry[]> {
  return (await listSuites(session)).map(mapSuite);
}

export async function fetchNativeSuite(session: SessionIdentity, suiteId: string): Promise<TestSuiteEntry | null> {
  return mapSuite(required((await client.suitesGet(session, { name: suiteId })).value, "Get Suite"));
}

/* === Private Functions === */

async function listSuites(session: SessionIdentity): Promise<Wire<Suite>[]> {
  return (await client.suitesList(session, {})).values;
}

/* === Entry Point === */
