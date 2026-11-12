// SPDX-License-Identifier: Apache-2.0

using Google.Protobuf;
using Grpc.Core;
using Harmos.V1;

namespace Stories.Support;

internal sealed class Wire(CallInvoker calls)
{
  internal static Method<TRequest, TResponse> Method<TRequest, TResponse>(MethodType type, string service, string name)
      where TRequest : IMessage<TRequest>, new() where TResponse : IMessage<TResponse>, new()
      => new(type, service, name,
          Marshallers.Create<TRequest>(value => value.ToByteArray(), bytes => (TRequest)new TRequest().Descriptor.Parser.ParseFrom(bytes)),
          Marshallers.Create<TResponse>(value => value.ToByteArray(), bytes => (TResponse)new TResponse().Descriptor.Parser.ParseFrom(bytes)));

  internal async Task<Description> DescribeAsync()
      => await calls.AsyncUnaryCall(Method<DescribeRequest, Description>(MethodType.Unary, "harmos.v1.Plugin", "Describe"),
          null, new CallOptions(), new DescribeRequest { Revision = 1 });

  internal async Task<T> InvokeAsync<T>(Description description, string operation, IMessage request, string scope = "",
      Func<ChildRequest, ByteString>? child = null) where T : IMessage<T>, new()
  {
    _ = description.Catalog.Operations.Single(item => item.Id == operation);
    using var call = calls.AsyncDuplexStreamingCall(Method<HostFrame, PluginFrame>(MethodType.DuplexStreaming, "harmos.v1.Plugin", "Invoke"), null, new CallOptions());
    await call.RequestStream.WriteAsync(new HostFrame
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
        await call.RequestStream.WriteAsync(new HostFrame { Child = new ChildResponse { Call = nested.Call, Payload = payload } });
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

  internal AsyncDuplexStreamingCall<PublishFrame, Samples> Publish()
      => calls.AsyncDuplexStreamingCall(Method<PublishFrame, Samples>(MethodType.DuplexStreaming, "harmos.v1.Plugin", "Publish"), null, new CallOptions());
}
