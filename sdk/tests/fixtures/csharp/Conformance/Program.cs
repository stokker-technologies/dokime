// SPDX-License-Identifier: Apache-2.0

using Harmos.Conformance;

var fixture = new Fixture();
await fixture.Build().ServeAsync(remote: args.Contains("--remote"));
