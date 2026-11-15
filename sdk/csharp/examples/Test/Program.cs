// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.Plugins;
using Step = Dokime.Plugins.Step;
using Suite = Dokime.Plugins.Suite;

// Compose a Suite and Template Through the Public Plugin Barrel.
var template = new Template("example.measurement", "Measurement")
    .Step(new Step("measure", "Measure", async context =>
        await context.LogAsync(Severity.Info, "Measurement Completed")));
var suite = new Suite("example", "Example", template);
var plugin = new Plugin("example", "1.0.0", suite)
    .Requires(HostOperation.EvidenceLogsAppend)
    .Requires(HostOperation.TimeGet);

// Serve at the Host-Assigned Endpoint; Remote Placement Remains User-Owned.
var addressIndex = Array.IndexOf(args, "--address");
var address = addressIndex < 0 ? null : args[addressIndex + 1];
await plugin.ServeAsync(address, remote: args.Contains("--remote"));
