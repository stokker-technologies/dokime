// SPDX-License-Identifier: Apache-2.0

using Google.Protobuf.WellKnownTypes;
using Harmos;
using Harmos.Plugins;
using Plugin = Dokime.Plugins.Plugin;
using HostOperation = Dokime.Plugins.HostOperation;
using Dokime.Plugins;

// Declare a Public Operation and Its Library Setting Through Public Namespaces.
var echo = new Operation<StringValue, StringValue>("example.echo", Visibility.Public);
var plugin = new Plugin("example.echo", "1.0.0")
    .Setting("prefix", new SettingValue { Text = "" })
    .Requires(HostOperation.PluginsSettingsList);
plugin.Build().Operation(echo, async (context, request) =>
    {
      var settings = await context.Host.Plugins.Settings.List();
      return new StringValue { Value = settings.Value.Values["prefix"].Text + request.Value };
    });
await plugin.ServeAsync(remote: args.Contains("--remote"));
