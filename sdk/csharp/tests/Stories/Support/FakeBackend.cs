// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.Clients;
using Dokime.Clients.ExecutionsResources;
using Google.Protobuf;
using Grpc.Core;
using Dokime.Clients.PluginsResources;
using Harmos;

namespace Stories.Support;

internal sealed class FakeBackend : CallInvoker
{
  private ExecutionDetail? execution;
  private readonly Dictionary<string, PluginEntry> plugins = [];
  private SettingValues settings = new();
  internal List<string> Calls { get; } = [];
  internal List<IMessage> Requests { get; } = [];
  internal List<ExecutionBinding> Bindings { get; } = [];
  internal DiscoveredContract? Operation
  {
    get; set;
  }
  internal bool EndBeforeSeal
  {
    get; set;
  }
  internal bool ChangeBindingOnStart
  {
    get; set;
  }
  internal int RegisteredCount => plugins.Count;
  internal string? RegistrationRefusal
  {
    get; set;
  }
  internal bool ConnectOnWatch
  {
    get; set;
  }

  private IMessage Reply(string method, IMessage request, CallOptions options)
  {
    Calls.Add(method);
    Requests.Add(request);
    if (options.Headers?.GetValue("authorization") != "Bearer test-token")
      throw new RpcException(new Grpc.Core.Status(StatusCode.Unauthenticated, "Call Fixture: Supply Credentials"));
    if (method == "/dokime.host.v1.Status/Get" && request is Harmos.Clients.StatusGetRequest)
      return new Harmos.Clients.StatusGetResponse();
    if (method == "/dokime.host.v1.Operations/Get" && request is Harmos.Clients.OperationsGetRequest operation)
    {
      if (Operation is null || Operation.Id != operation.Id)
        throw new RpcException(new Grpc.Core.Status(StatusCode.NotFound, "Get Operation: Register the Fixture Contract"));
      return new Harmos.Clients.OperationsGetResponse { Operation = new DiscoveredOperation { Contract = Operation.Clone() } };
    }
    if (method == "/dokime.host.v1.Invocations/Create" && request is Harmos.Clients.InvocationsCreateRequest invocation)
    {
      if (Operation is null || Operation.Id != invocation.Operation || Operation.Fingerprint != invocation.Fingerprint)
        throw new RpcException(new Grpc.Core.Status(StatusCode.InvalidArgument, "Invoke Operation: Select the Fixture Contract"));
      return new Harmos.Clients.InvocationsCreateResponse { Payload = invocation.Payload };
    }
    if (request is CasesGetRequest selected)
    {
      if (selected.Name.Length == 0)
        throw new RpcException(new Grpc.Core.Status(StatusCode.InvalidArgument, "Get Case: Supply a Name"));
      return new CasesGetResponse { Value = new CaseDetail { Summary = new CaseSummary { CaseId = selected.Name, SuiteId = "capture", TemplateId = "capture.confirmation" } } };
    }
    if (request is CyclesCreateRequest cycle)
      return new CyclesCreateResponse { Cycle = new CycleSummary { CycleId = "cycle", Metadata = cycle.Metadata } };
    if (request is ExecutionsCreateRequest created)
    {
      Bindings.AddRange(created.Executions.Select(binding => binding.Clone()));
      var binding = created.Executions.Single();
      execution = new ExecutionDetail { CycleId = created.Parent, Summary = new ExecutionSummary { ExecutionId = binding.ExecutionId, CaseId = binding.CaseId } };
      return new ExecutionsCreateResponse { Executions = { execution.Clone() } };
    }
    if (request is ExecutionsStartRequest)
    {
      var detail = execution!.Clone();
      if (ChangeBindingOnStart)
        detail.CycleId = "different-cycle";
      return new ExecutionsStartResponse { Execution = detail };
    }
    if (request is ExecutionsGetRequest)
      return new ExecutionsGetResponse { Execution = execution!.Clone() };
    if (request is ExecutionsSealRequest)
      return new ExecutionsSealResponse { Outcome = new ExecutionOutcome { Evaluated = new EvaluatedOutcome { Status = ResultStatus.Passed } } };
    if (method == "/dokime.host.v1.Plugins/Create" && request is PluginsCreateRequest registration)
    {
      if (RegistrationRefusal is not null)
        throw new RpcException(new Grpc.Core.Status(StatusCode.FailedPrecondition, RegistrationRefusal));
      var entry = new PluginEntry
      {
        Name = registration.Name,
        Placement = registration.Placement.Clone(),
        Value = registration.Value.Clone(),
        Origin = registration.Origin,
        State = ConnectOnWatch ? PluginState.Connecting : PluginState.Ready,
        Revision = ByteString.CopyFromUtf8("revision")
      };
      // Like the Product Host, an Existing Name Reuses Its Entry.
      plugins[entry.Name] = entry;
      return new PluginsCreateResponse { Entry = entry.Clone() };
    }
    if (method == "/dokime.host.v1.Plugins/Get" && request is PluginsGetRequest selectedPlugin)
      return new PluginsGetResponse { Entry = plugins[selectedPlugin.Name].Clone() };
    if (method == "/dokime.host.v1.Plugins/List" && request is PluginsListRequest)
      return new PluginsListResponse { Entries = { plugins.Values.Select(entry => entry.Clone()) } };
    if (method == "/dokime.host.v1.Plugins/Set" && request is PluginsSetRequest updatedPlugin)
    {
      var entry = plugins[updatedPlugin.Name];
      entry.Value = updatedPlugin.Value.Clone();
      return new PluginsSetResponse { Entry = entry.Clone() };
    }
    if (method == "/dokime.host.v1.Plugins/Restart" && request is PluginsRestartRequest restartedPlugin)
    {
      var entry = plugins[restartedPlugin.Name];
      entry.Activation++;
      return new PluginsRestartResponse { Entry = entry.Clone() };
    }
    if (method == "/dokime.host.v1.Plugins/Replace" && request is PluginsReplaceRequest replacedPlugin)
    {
      var entry = plugins[replacedPlugin.Name];
      entry.Placement = replacedPlugin.Placement.Clone();
      return new PluginsReplaceResponse { Entry = entry.Clone() };
    }
    if (method == "/dokime.host.v1.plugins.Settings/List" && request is SettingsListRequest)
      return new SettingsListResponse { Value = settings.Clone(), Revision = ByteString.CopyFromUtf8("settings-revision") };
    if (method == "/dokime.host.v1.plugins.Settings/Set" && request is Dokime.Clients.PluginsResources.SettingsSetRequest updatedSettings)
    {
      settings = updatedSettings.Value.Clone();
      return new Dokime.Clients.PluginsResources.SettingsSetResponse { Value = settings.Clone(), Revision = ByteString.CopyFromUtf8("settings-revision") };
    }
    if (method == "/dokime.host.v1.Plugins/Delete" && request is PluginsDeleteRequest deletedPlugin)
    {
      plugins.Remove(deletedPlugin.Name);
      return new PluginsDeleteResponse { Receipt = new Receipt() };
    }
    throw new RpcException(new Grpc.Core.Status(StatusCode.Unimplemented, $"Call {method}: Add the Fixture Operation"));
  }

  public override AsyncUnaryCall<TResponse> AsyncUnaryCall<TRequest, TResponse>(Method<TRequest, TResponse> method, string? host, CallOptions options, TRequest request)
  {
    Task<TResponse> response;
    try
    {
      var reply = Reply(method.FullName, (IMessage)request, options);
      response = Task.FromResult(method.ResponseMarshaller.Deserializer(reply.ToByteArray()));
    }
    catch (RpcException error)
    {
      response = Task.FromException<TResponse>(error);
    }
    return new(response, Task.FromResult(new Grpc.Core.Metadata()), () => Grpc.Core.Status.DefaultSuccess, () => new Grpc.Core.Metadata(), () => { });
  }

  public override AsyncServerStreamingCall<TResponse> AsyncServerStreamingCall<TRequest, TResponse>(Method<TRequest, TResponse> method, string? host, CallOptions options, TRequest request)
  {
    Calls.Add(method.FullName);
    Requests.Add((IMessage)request);
    if (method.FullName == "/dokime.host.v1.Plugins/Watch" && request is PluginsWatchRequest watchedPlugin)
    {
      var entries = plugins.Values.Where(entry => !watchedPlugin.HasName || entry.Name == watchedPlugin.Name).ToArray();
      var snapshot = new PluginsWatchResponse { Snapshot = new PluginSnapshot { Entries = { entries.Select(entry => entry.Clone()) } } };
      var updates = new List<PluginsWatchResponse> { snapshot };
      if (ConnectOnWatch)
      {
        foreach (var entry in entries)
        {
          entry.State = PluginState.Ready;
          updates.Add(new PluginsWatchResponse { Change = new PluginChange { Entry = entry.Clone() } });
        }
      }
      var observed = updates.Select(frame => method.ResponseMarshaller.Deserializer(frame.ToByteArray())).ToArray();
      return new(new Reader<TResponse>(observed), Task.FromResult(new Grpc.Core.Metadata()), () => Grpc.Core.Status.DefaultSuccess, () => new Grpc.Core.Metadata(), () => { });
    }
    var frames = new List<ProgressWatchResponse>
    {
      new() { Phase = "Running", Value = new ExecutionProgressValue() }
    };
    if (!EndBeforeSeal)
      frames.Add(new ProgressWatchResponse
      {
        Phase = "Sealed",
        Sealed = true,
        Value = new ExecutionProgressValue { Completed = 1, Total = 1 },
        Outcome = new ExecutionOutcome { Evaluated = new EvaluatedOutcome { Status = ResultStatus.Passed } }
      });
    var values = frames.Select(frame => method.ResponseMarshaller.Deserializer(frame.ToByteArray())).ToArray();
    return new(new Reader<TResponse>(values), Task.FromResult(new Grpc.Core.Metadata()), () => Grpc.Core.Status.DefaultSuccess, () => new Grpc.Core.Metadata(), () => { });
  }

  private sealed class Reader<T>(IReadOnlyList<T> values) : IAsyncStreamReader<T>
  {
    private int index = -1;
    public T Current => values[index];
    public Task<bool> MoveNext(CancellationToken cancellationToken)
    {
      cancellationToken.ThrowIfCancellationRequested();
      index++;
      return Task.FromResult(index < values.Count);
    }
  }
  public override TResponse BlockingUnaryCall<TRequest, TResponse>(Method<TRequest, TResponse> method, string? host, CallOptions options, TRequest request) => throw new NotSupportedException();
  public override AsyncClientStreamingCall<TRequest, TResponse> AsyncClientStreamingCall<TRequest, TResponse>(Method<TRequest, TResponse> method, string? host, CallOptions options) => throw new NotSupportedException();
  public override AsyncDuplexStreamingCall<TRequest, TResponse> AsyncDuplexStreamingCall<TRequest, TResponse>(Method<TRequest, TResponse> method, string? host, CallOptions options) => throw new NotSupportedException();
}
