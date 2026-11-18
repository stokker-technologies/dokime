// SPDX-License-Identifier: Apache-2.0

using Dokime.Authoring;
using Dokime.Plugins;

var echo = new Operation<string, string>("example.echo", Visibility.Public);
var plugin = new Plugin("example.echo", "1.0.0")
    .Operation(echo, (_, request) => Task.FromResult(request));
await plugin.ServeAsync(remote: args.Contains("--remote"));
