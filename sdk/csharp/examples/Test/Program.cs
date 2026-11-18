// SPDX-License-Identifier: Apache-2.0

using Dokime.Plugins;

var addressIndex = Array.IndexOf(args, "--address");
var address = addressIndex < 0 ? null : args[addressIndex + 1];
await Capture.Plugin.ServeAsync(address, remote: args.Contains("--remote"));
