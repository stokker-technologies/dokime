// SPDX-License-Identifier: Apache-2.0

using Google.Protobuf;
using Grpc.Core;
using Harmos;

namespace Dokime;

internal sealed class TransportInvoker(CallInvoker transport) : CallInvoker
{
  private static OperationException FromTransport(string operation, RpcException error, string? requestId)
  {
    var detail = error.Trailers.FirstOrDefault(entry => entry.Key == "grpc-status-details-bin");
    if (detail is not null)
    {
      try
      {
        var remote = Failure.Parser.ParseFrom(detail.ValueBytes);
        if (remote.Classification != Classification.Unspecified)
        {
          if (requestId is not null)
            remote.Details["request_id"] = requestId;
          return new OperationException(remote);
        }
      }
      catch (InvalidProtocolBufferException)
      {
      }
    }
    var classification = error.StatusCode switch
    {
      StatusCode.Cancelled => Classification.Cancelled,
      StatusCode.DeadlineExceeded => Classification.TimedOut,
      StatusCode.Unavailable => Classification.Transient,
      StatusCode.Aborted or StatusCode.FailedPrecondition or StatusCode.PermissionDenied => Classification.Refused,
      _ => Classification.Permanent
    };
    var failure = new Failure
    {
      Classification = classification,
      Code = error.StatusCode.ToString(),
      Message = error.Status.Detail,
      Action = "Check the Host Failure and Retry the Unchanged Request When Appropriate",
      Operation = operation
    };
    if (requestId is not null)
      failure.Details["request_id"] = requestId;
    return new OperationException(failure);
  }

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
      throw FromTransport(operation, error, identity);
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
    return new AsyncServerStreamingCall<TResponse>(new ResponseReader<TResponse>(call.ResponseStream, method.FullName), call.ResponseHeadersAsync,
      call.GetStatus, call.GetTrailers, call.Dispose);
  }
  public override TResponse BlockingUnaryCall<TRequest, TResponse>(Method<TRequest, TResponse> method, string? host, CallOptions options, TRequest request) =>
    throw new NotSupportedException("Call SDK: Use the Asynchronous Client");
  public override AsyncClientStreamingCall<TRequest, TResponse> AsyncClientStreamingCall<TRequest, TResponse>(Method<TRequest, TResponse> method, string? host, CallOptions options) =>
    throw new NotSupportedException("Call SDK: Select a Declared Unary Operation");
  public override AsyncDuplexStreamingCall<TRequest, TResponse> AsyncDuplexStreamingCall<TRequest, TResponse>(Method<TRequest, TResponse> method, string? host, CallOptions options) =>
    throw new NotSupportedException("Call SDK: Select a Declared Unary Operation or Server Watch");

  private sealed class ResponseReader<T>(IAsyncStreamReader<T> reader, string operation) : IAsyncStreamReader<T>
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
        throw FromTransport(operation, error, null);
      }
    }
  }
}
