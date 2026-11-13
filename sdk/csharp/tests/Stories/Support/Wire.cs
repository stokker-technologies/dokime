// SPDX-License-Identifier: Apache-2.0

using Google.Protobuf;
using Grpc.Core;
using Harmos.V1;
using Harmos.Plugin.V1;

namespace Stories.Support;

internal sealed class Wire(CallInvoker calls)
{
  internal static Method<TRequest, TResponse> Method<TRequest, TResponse>(MethodType type, string service, string name)
      where TRequest : IMessage<TRequest>, new() where TResponse : IMessage<TResponse>, new()
      => new(type, service, name,
          Marshallers.Create<TRequest>(value => value.ToByteArray(), bytes => (TRequest)new TRequest().Descriptor.Parser.ParseFrom(bytes)),
          Marshallers.Create<TResponse>(value => value.ToByteArray(), bytes => (TResponse)new TResponse().Descriptor.Parser.ParseFrom(bytes)));

  internal async Task<InstanceGetResponse> DescribeAsync()
      => await calls.AsyncUnaryCall(Method<InstanceGetRequest, InstanceGetResponse>(MethodType.Unary, "harmos.plugin.v1.Instance", "Get"),
          null, new CallOptions(), new InstanceGetRequest { Revision = 1 });

  internal async Task<T> InvokeAsync<T>(InstanceGetResponse description, string operation, IMessage request, string scope = "",
      Func<ChildRequest, ByteString>? child = null) where T : IMessage<T>, new()
  {
    _ = description.Catalog.Operations.Single(item => item.Id == operation);
    using var call = calls.AsyncDuplexStreamingCall(Method<InstanceInvokeRequest, InstanceInvokeResponse>(MethodType.DuplexStreaming, "harmos.plugin.v1.Instance", "Invoke"), null, new CallOptions());
    await call.RequestStream.WriteAsync(new InstanceInvokeRequest
    {
      Request = new Request
      {
        Operation = operation,
        Payload = request.ToByteString(),
        Invocation = Guid.NewGuid().ToString(),
        Attempt = 1,
        Scope = scope
      }
    });
    while (await call.ResponseStream.MoveNext())
    {
      var frame = call.ResponseStream.Current;
      if (frame.Child is { } nested)
      {
        var payload = child?.Invoke(nested) ?? throw new InvalidOperationException($"Unexpected child call: {nested.Operation}");
        await call.RequestStream.WriteAsync(new InstanceInvokeRequest { Child = new ChildResponse { Call = nested.Call, Payload = payload } });
      }
      if (frame.Response is { } response)
      {
        await call.RequestStream.CompleteAsync();
        if (response.Failure is { } failure) throw new Harmos.Sdk.PluginFailureException(failure);
        return (T)new T().Descriptor.Parser.ParseFrom(response.Payload);
      }
    }
    throw new InvalidOperationException("Missing unary operation response");
  }

  internal AsyncDuplexStreamingCall<InstanceSamplesRequest, InstanceSamplesResponse> Publish()
      => calls.AsyncDuplexStreamingCall(Method<InstanceSamplesRequest, InstanceSamplesResponse>(MethodType.DuplexStreaming, "harmos.plugin.v1.Instance", "Samples"), null, new CallOptions());
}
