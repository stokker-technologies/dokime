// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.Protocol.V1;

var template = new Template("example.measurement", "Measurement")
    .Step(new Dokime.Step("measure", "Measure", async context =>
        await context.LogAsync(Severity.Info, "Measurement completed")));
var suite = new Dokime.Suite("example", "Example", template);
var plugin = new Dokime.Plugin("example", "1.0.0", suite).Requires(Host.Log);
var addressIndex = Array.IndexOf(args, "--address");
var address = addressIndex < 0 ? null : args[addressIndex + 1];
await plugin.ServeAsync(address, remote: args.Contains("--remote"));
