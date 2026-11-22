// SPDX-License-Identifier: Apache-2.0

using System.Collections.ObjectModel;
using Google.Protobuf;
using Grpc.Core;

namespace Dokime;

public sealed record Failure(string Classification, string Code, string Message, string Action, string Operation = "", string Plugin = "",
  Failure? Cause = null, IReadOnlyDictionary<string, string>? Details = null, string Diagnostics = "")
{
  internal static Failure FromWire(Harmos.Failure value) => new(value.Classification.ToString(), value.Code, value.Message, value.Action,
    value.Operation, value.Plugin, value.Cause is null ? null : FromWire(value.Cause), new ReadOnlyDictionary<string, string>(new Dictionary<string, string>(value.Details)), value.Diagnostics);
  internal Harmos.Failure ToWire()
  {
    var value = new Harmos.Failure
    {
      Classification = Enum.TryParse<Harmos.Classification>(Classification.Replace(" ", ""), out var classification) ? classification : Harmos.Classification.Permanent,
      Code = Code,
      Message = Message,
      Action = Action,
      Operation = Operation,
      Plugin = Plugin,
      Cause = Cause?.ToWire(),
      Diagnostics = Diagnostics
    };
    foreach (var (key, detail) in Details ?? new Dictionary<string, string>())
      value.Details.Add(key, detail);
    return value;
  }
}

public class SdkError : Exception
{
  public Failure Failure
  {
    get;
  }
  public string? RequestId
  {
    get;
  }
  public SdkError(Failure failure, string? requestId = null, Exception? cause = null) : base($"{failure.Message}; {failure.Action}", cause)
  {
    var details = new Dictionary<string, string>(failure.Details ?? new Dictionary<string, string>());
    if (requestId is not null)
      details["request_id"] = requestId;
    Failure = failure with
    {
      Details = new ReadOnlyDictionary<string, string>(details)
    };
    RequestId = requestId;
  }
  internal static SdkError FromTransport(string operation, RpcException error, string? requestId)
  {
    var detail = error.Trailers.FirstOrDefault(entry => entry.Key == "grpc-status-details-bin");
    if (detail is not null)
    {
      try
      {
        var failure = Harmos.Failure.Parser.ParseFrom(detail.ValueBytes);
        if (failure.Classification != Harmos.Classification.Unspecified)
          return new SdkError(Failure.FromWire(failure), requestId, error);
      }
      catch (InvalidProtocolBufferException)
      {
      }
    }
    var classification = error.StatusCode switch
    {
      StatusCode.Cancelled => "Cancelled",
      StatusCode.DeadlineExceeded => "Timed Out",
      StatusCode.Unavailable => "Transient",
      StatusCode.Aborted or StatusCode.FailedPrecondition or StatusCode.PermissionDenied => "Refused",
      _ => "Permanent"
    };
    return new SdkError(new Failure(classification, error.StatusCode.ToString(), error.Status.Detail,
      "Check the Host Failure and Retry the Unchanged Request When Appropriate", operation), requestId, error);
  }
}


internal sealed class FailureInvoker(CallInvoker transport) : CallInvoker
{
  private static string? Identity<T>(T request) => request is IMessage message
    && message.Descriptor.FindFieldByName("request_id") is { } field ? (string)field.Accessor.GetValue(message) : null;

  private static async Task<T> ResponseAsync<T>(Task<T> response, string operation, string? identity)
  {
    try
    {
      return await response;
    }
    catch (RpcException error)
    {
      throw SdkError.FromTransport(operation, error, identity);
    }
  }

  public override AsyncUnaryCall<TResponse> AsyncUnaryCall<TRequest, TResponse>(Method<TRequest, TResponse> method, string? host, CallOptions options, TRequest request)
  {
    var call = transport.AsyncUnaryCall(method, host, options, request);
    return new AsyncUnaryCall<TResponse>(ResponseAsync(call.ResponseAsync, method.FullName, Identity(request)), call.ResponseHeadersAsync,
      call.GetStatus, call.GetTrailers, call.Dispose);
  }
  public override AsyncServerStreamingCall<TResponse> AsyncServerStreamingCall<TRequest, TResponse>(Method<TRequest, TResponse> method, string? host, CallOptions options, TRequest request)
  {
    var call = transport.AsyncServerStreamingCall(method, host, options, request);
    return new AsyncServerStreamingCall<TResponse>(new FailureReader<TResponse>(call.ResponseStream, method.FullName), call.ResponseHeadersAsync,
      call.GetStatus, call.GetTrailers, call.Dispose);
  }
  public override TResponse BlockingUnaryCall<TRequest, TResponse>(Method<TRequest, TResponse> method, string? host, CallOptions options, TRequest request) =>
    throw new NotSupportedException("Call SDK: Use the Asynchronous Client");
  public override AsyncClientStreamingCall<TRequest, TResponse> AsyncClientStreamingCall<TRequest, TResponse>(Method<TRequest, TResponse> method, string? host, CallOptions options) =>
    throw new NotSupportedException("Call SDK: Select a Declared Unary Operation");
  public override AsyncDuplexStreamingCall<TRequest, TResponse> AsyncDuplexStreamingCall<TRequest, TResponse>(Method<TRequest, TResponse> method, string? host, CallOptions options) =>
    throw new NotSupportedException("Call SDK: Select a Declared Unary Operation or Server Watch");

  private sealed class FailureReader<T>(IAsyncStreamReader<T> reader, string operation) : IAsyncStreamReader<T>
  {
    public T Current => reader.Current;
    public async Task<bool> MoveNext(CancellationToken cancellationToken)
    {
      try
      {
        return await reader.MoveNext(cancellationToken);
      }
      catch (RpcException error) when (error.StatusCode == StatusCode.Cancelled && cancellationToken.IsCancellationRequested)
      {
        throw new OperationCanceledException("Observe SDK: Observation Was Cancelled", error, cancellationToken);
      }
      catch (RpcException error)
      {
        throw SdkError.FromTransport(operation, error, null);
      }
    }
  }
}
