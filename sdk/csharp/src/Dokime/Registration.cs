/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

using System.Collections.Concurrent;
using System.Reflection;
using System.Runtime.CompilerServices;
using Dokime.Clients;
using Harmos;
using Harmos.Plugins;

namespace Dokime;

public sealed class PluginCollection
{
  private readonly Client client;
  private readonly CancellationTokenSource closing = new();
  private readonly ConcurrentDictionary<long, Task<RegisteredPlugin>> pending = new();
  private long nextRegistration;
  // The Product SDK Version This Plugin Was Built With, Without Build Metadata.
  private static readonly string SdkVersion = typeof(Client).Assembly.GetCustomAttribute<AssemblyInformationalVersionAttribute>()!.InformationalVersion.Split('+')[0];
  internal List<RegisteredPlugin> Owned { get; } = [];
  internal List<TemplateBinding> Bindings { get; } = [];
  internal PluginCollection(Client client) => this.client = client;

  public async Task<RegisteredPlugin> RegisterAsync(Plugin plugin, ConnectionSettings? settings = null, string? requestId = null,
    CancellationToken cancellationToken = default)
  {
    using var cancellation = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken, closing.Token);
    cancellation.Token.ThrowIfCancellationRequested();
    var identity = Interlocked.Increment(ref nextRegistration);
    var registration = RegisterCoreAsync(plugin, settings, requestId, cancellation.Token);
    pending.TryAdd(identity, registration);
    try
    {
      return await registration;
    }
    finally
    {
      pending.TryRemove(identity, out _);
    }
  }

  internal async Task StopRegistrationsAsync()
  {
    await closing.CancelAsync();
    List<Exception> failures = [];
    foreach (var registration in pending.Values)
    {
      try
      {
        await registration;
      }
      catch (OperationCanceledException)
      {
      }
      catch (OperationException failure) when (closing.IsCancellationRequested && failure.Failure.Classification == Harmos.Classification.Cancelled)
      {
      }
      catch (Exception failure)
      {
        failures.Add(failure);
      }
    }
    if (failures.Count > 0)
      throw new AggregateException("Close Client: Resolve Each Pending Registration Failure", failures);
  }

  private async Task<RegisteredPlugin> RegisterCoreAsync(Plugin plugin, ConnectionSettings? settings, string? requestId,
    CancellationToken cancellationToken)
  {
    _ = client.Options(cancellationToken);
    var address = settings?.PluginAddress ?? "127.0.0.1:0";
    var endpoint = await PluginEndpoint.StartAsync(plugin, address, cancellationToken);
    RegisteredPlugin? registered = null;
    try
    {
      var placement = new PluginPlacement { Remote = new PluginRemote { Url = endpoint.Address } };
      var reply = await client.Transport.Plugins.Create(plugin.Identifier, placement, new PluginValue { Enabled = true }, Client.Identity(requestId), sdkVersion: SdkVersion, context: client.Options(cancellationToken));
      registered = new RegisteredPlugin(client, this, reply.Entry, endpoint);
      Owned.Add(registered);
      registered.CheckFailure();
      if (registered.State != "Ready")
      {
        await foreach (var frame in client.Transport.Plugins.Watch(plugin.Identifier, context: client.Options(cancellationToken), cancellationToken: cancellationToken))
        {
          if (frame.ResyncRequired.Length > 0)
            throw new InvalidDataException("Register Plugin: Reconnect After Plugin Resynchronization");
          var entries = frame.Snapshot?.Entries.AsEnumerable() ?? (frame.Change?.Entry is { } entry ? [entry] : []);
          foreach (var observed in entries.Where(entry => entry.Name == plugin.Identifier))
          {
            registered.Observe(observed);
            registered.CheckFailure();
            if (registered.State == "Ready")
              break;
          }
          if (registered.State == "Ready")
            break;
        }
      }
      if (registered.State != "Ready")
        throw new InvalidDataException("Register Plugin: Reconnect; Readiness Observation Ended Before Ready");
      foreach (var suiteType in plugin.Suites)
      {
        var suite = (Suite)Activator.CreateInstance(suiteType)!;
        foreach (var type in suite.Templates)
        {
          var template = (Templates.TestTemplate)Activator.CreateInstance(type)!;
          var identifier = $"{suite.Identifier}.{template.Identifier}";
          var contract = plugin.Adapter.Suites.Single(value => value.Metadata.Identifier == suite.Identifier).Templates.Single(value => value.Definition.Metadata.Identifier == identifier);
          Bindings.Add(new TemplateBinding(type, registered.Identifier, suite.Identifier, identifier, contract));
        }
      }
      return registered;
    }
    catch (Exception failure)
    {
      // A Failed Registration Stops Only Its Listener; an Existing Entry Stays for the Next Start.
      try
      {
        if (registered is not null)
          await registered.ReleaseAsync();
        else
          await endpoint.DisposeAsync();
      }
      catch (Exception cleanup)
      {
        throw new AggregateException("Register Plugin: Resolve the Registration and Cleanup Failures", failure, cleanup);
      }
      throw;
    }
  }

  public Task<RegisteredPlugin> UploadAsync(string bundle, string? requestId = null, CancellationToken cancellationToken = default)
  {
    throw new InvalidOperationException("Upload Plugin: Publish the Product Managed-Bundle Installation Contract and Regenerate Bindings");
  }

  public async Task<RegisteredPlugin> GetAsync(string identifier, CancellationToken cancellationToken = default)
  {
    var reply = await client.Transport.Plugins.Get(identifier, context: client.Options(cancellationToken));
    return new RegisteredPlugin(client, this, reply.Entry);
  }

  public async IAsyncEnumerable<RegisteredPlugin> ListAsync([EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var reply = await client.Transport.Plugins.List(context: client.Options(cancellationToken));
    foreach (var entry in reply.Entries)
      yield return new RegisteredPlugin(client, this, entry);
  }
}

public sealed class RegisteredPlugin : IAsyncDisposable
{
  private readonly Client client;
  private readonly PluginCollection owner;
  private readonly PluginEndpoint? endpoint;
  private PluginEntry entry;
  private bool unregistered;
  internal RegisteredPlugin(Client client, PluginCollection owner, PluginEntry entry, PluginEndpoint? endpoint = null)
  {
    this.client = client;
    this.owner = owner;
    this.entry = entry.Clone();
    this.endpoint = endpoint;
  }
  public string Identifier => entry.Name;
  public string State => entry.State.ToString();
  internal Task WaitAsync(CancellationToken cancellationToken) => endpoint?.Serving.WaitAsync(cancellationToken)
    ?? throw new InvalidOperationException("Wait for Plugin: Register an Owned Listener");
  internal void Observe(PluginEntry observed)
  {
    if (observed.Name != Identifier)
      throw new InvalidDataException("Read Plugin: Reconnect; the Host Changed the Bound Identity");
    entry = observed.Clone();
  }
  internal void CheckFailure()
  {
    if (entry.Failure is not null)
      throw new OperationException(entry.Failure);
  }
  public async Task<RegisteredPlugin> RefreshAsync(CancellationToken cancellationToken = default)
  {
    var reply = await client.Transport.Plugins.Get(Identifier, context: client.Options(cancellationToken));
    Observe(reply.Entry);
    return this;
  }
  public async Task UnregisterAsync(CancellationToken cancellationToken = default)
  {
    if (unregistered)
      return;
    var owned = owner.Owned.FirstOrDefault(value => value.Identifier == Identifier);
    if (owned is not null && !ReferenceEquals(owned, this))
    {
      await owned.UnregisterAsync(cancellationToken);
      unregistered = true;
      return;
    }
    var reply = await client.Transport.Plugins.Get(Identifier, context: client.CleanupOptions(cancellationToken));
    if (endpoint is not null && reply.Entry.Placement?.Remote?.Url != endpoint.Address)
      throw new InvalidOperationException("Unregister Plugin: Resolve the Changed Placement Before Removing This Client's Registration");
    await client.Transport.Plugins.Delete(Identifier, reply.Entry.Revision, client.CleanupOptions(cancellationToken));
    await ReleaseAsync();
    unregistered = true;
  }
  internal async Task ReleaseAsync()
  {
    if (endpoint is not null)
      await endpoint.DisposeAsync();
    owner.Owned.Remove(this);
    owner.Bindings.RemoveAll(binding => binding.Plugin == Identifier);
  }
  public async ValueTask DisposeAsync() => await ReleaseAsync();
}

internal sealed record TemplateBinding(Type Type, string Plugin, string Suite, string Template, Templates.Template Contract);

internal sealed class PluginEndpoint(Listener listener) : IAsyncDisposable
{
  private Task? closing;
  internal Task Serving { get; } = listener.WaitAsync();
  internal string Address => "http://" + listener.Address;
  internal static async Task<PluginEndpoint> StartAsync(Plugin plugin, string address, CancellationToken cancellationToken)
  {
    var listener = await plugin.Adapter.Build().StartAsync(address, remote: true, cancellationToken);
    return new PluginEndpoint(listener);
  }
  public ValueTask DisposeAsync() => new(closing ??= listener.DisposeAsync().AsTask());
}
