// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.Clients;
using Dokime.Clients.ExecutionsResources;
using Google.Protobuf;
using Grpc.Core;
using Harmos.Clients;

namespace Stories.Support;

internal sealed class FakeBackend : CallInvoker
{
  private ExecutionDetail? execution;
  private readonly Dictionary<string, PluginEntry> plugins = [];
  internal List<string> Calls { get; } = [];
  internal List<ExecutionBinding> Bindings { get; } = [];
  internal bool EndBeforeSeal
  {
    get; set;
  }
  internal bool ChangeBindingOnStart
  {
    get; set;
  }
  internal int RegisteredCount => plugins.Count;

  private static T Field<T>(IMessage request, string name) => (T)request.Descriptor.FindFieldByName(name).Accessor.GetValue(request);

  private IMessage Reply(string method, IMessage request, CallOptions options)
  {
    Calls.Add(method);
    if (options.Headers?.GetValue("authorization") != "Bearer test-token")
      throw new RpcException(new Grpc.Core.Status(StatusCode.Unauthenticated, "Call Fixture: Supply Credentials"));
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
    if (method == "/harmos.host.v1.Plugins/Create")
    {
      var entry = new PluginEntry { Name = Field<string>(request, "name"), Placement = Field<PluginPlacement>(request, "placement"), State = PluginState.Ready, Revision = ByteString.CopyFromUtf8("revision") };
      plugins.Add(entry.Name, entry);
      return new PluginsCreateResponse { Entry = entry };
    }
    if (method == "/harmos.host.v1.Plugins/Get")
      return new PluginsGetResponse { Entry = plugins[Field<string>(request, "name")].Clone() };
    if (method == "/harmos.host.v1.Plugins/Delete")
    {
      plugins.Remove(Field<string>(request, "name"));
      return new PluginsDeleteResponse { Receipt = new Harmos.Receipt() };
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
    var frames = new List<ProgressWatchResponse>
    {
      new() { Phase = "Running", Value = new ExecutionProgressValue() }
    };
    if (!EndBeforeSeal)
      frames.Add(new ProgressWatchResponse { Phase = "Sealed", Sealed = true,
        Value = new ExecutionProgressValue { Completed = 1, Total = 1 },
        Outcome = new ExecutionOutcome { Evaluated = new EvaluatedOutcome { Status = ResultStatus.Passed } } });
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
