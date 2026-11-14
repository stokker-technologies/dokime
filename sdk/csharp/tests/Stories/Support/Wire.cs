// SPDX-License-Identifier: Apache-2.0

using System.Collections;
using Google.Protobuf;
using Google.Protobuf.Reflection;
using Google.Protobuf.WellKnownTypes;
using Grpc.Core;
using Harmos;
using Harmos.Plugins;
using TransportPlugin = Harmos.Plugins.Plugin;

namespace Stories.Support;

// Test the Actual Internal Carrier Through Its Descriptors Without Exporting Wire Types.
internal sealed class Wire(CallInvoker calls)
{
  private static readonly FileDescriptor Instance = (FileDescriptor)typeof(TransportPlugin).Assembly
      .GetType("Harmos.Internal.Plugin.InstanceReflection", throwOnError: true)!
      .GetProperty("Descriptor")!.GetValue(null)!;

  internal sealed record Description(string Id, Timestamp StartedAt, Declarations Declarations);
  internal sealed record Child(string Operation, ByteString Payload);
  internal sealed record Sample(Timestamp At, ByteString Row);

  private static MessageDescriptor Descriptor(string name)
  {
    List<FileDescriptor> pending = [Instance];
    HashSet<string> visited = [];
    for (var index = 0; index < pending.Count; index++)
    {
      var file = pending[index];
      if (!visited.Add(file.Name))
        continue;
      var message = file.MessageTypes.FirstOrDefault(item => item.Name == name);
      if (message is not null)
        return message;
      pending.AddRange(file.Dependencies);
    }
    throw new InvalidOperationException($"Read Carrier Descriptor: Missing {name}");
  }

  private static IMessage Frame(string name, params (string Name, object Value)[] fields)
  {
    var descriptor = Descriptor(name);
    var message = descriptor.Parser.ParseFrom(ByteString.Empty);
    foreach (var (field, value) in fields)
      descriptor.FindFieldByName(field).Accessor.SetValue(message, value);
    return message;
  }

  private static T Field<T>(IMessage message, string name)
      => (T)message.Descriptor.FindFieldByName(name).Accessor.GetValue(message);

  private static Method<ByteString, ByteString> Method(MethodType type, string name)
      => new(type, Instance.Services.Single().FullName, name,
          Marshallers.Create<ByteString>(value => value.ToByteArray(), ByteString.CopyFrom),
          Marshallers.Create<ByteString>(value => value.ToByteArray(), ByteString.CopyFrom));

  internal async Task<Description> DescribeAsync()
  {
    using var call = calls.AsyncUnaryCall(Method(MethodType.Unary, "Get"), null, new CallOptions(),
        Frame("InstanceGetRequest", ("revision", 1u)).ToByteString());
    var response = Descriptor("InstanceGetResponse").Parser.ParseFrom(await call.ResponseAsync);
    return new Description(Field<string>(response, "id"), Field<Timestamp>(response, "started_at"),
        Field<Declarations>(response, "declarations"));
  }

  internal async Task<T> InvokeAsync<T>(Description description, string operation, IMessage request, string scope = "",
      Func<Child, ByteString>? child = null) where T : IMessage<T>, new()
  {
    var contract = description.Declarations.Operations.Single(item => item.Id == operation);
    if (contract.Request != request.Descriptor.FullName || contract.Response != new T().Descriptor.FullName)
      throw new InvalidOperationException($"Invoke {operation}: Payload Types Differ from the Frozen Contract");
    using var call = calls.AsyncDuplexStreamingCall(Method(MethodType.DuplexStreaming, "Invoke"), null, new CallOptions());
    var opening = Frame("Request", ("operation", operation), ("payload", request.ToByteString()),
        ("invocation", Guid.NewGuid().ToString()), ("attempt", 1u), ("scope", scope));
    await call.RequestStream.WriteAsync(Frame("InstanceInvokeRequest", ("request", opening)).ToByteString());
    while (await call.ResponseStream.MoveNext())
    {
      var frame = Descriptor("InstanceInvokeResponse").Parser.ParseFrom(call.ResponseStream.Current);
      var nested = Field<IMessage?>(frame, "child");
      if (nested is not null)
      {
        var payload = child?.Invoke(new Child(Field<string>(nested, "operation"), Field<ByteString>(nested, "payload")))
            ?? throw new InvalidOperationException("Invoke: Unexpected Child Call");
        var response = Frame("ChildResponse", ("call", Field<ulong>(nested, "call")), ("payload", payload));
        await call.RequestStream.WriteAsync(Frame("InstanceInvokeRequest", ("child", response)).ToByteString());
      }
      var terminal = Field<IMessage?>(frame, "response");
      if (terminal is null)
        continue;
      await call.RequestStream.CompleteAsync();
      var failure = Field<Failure?>(terminal, "failure");
      if (failure is not null)
        throw new PluginFailureException(failure);
      return (T)new T().Descriptor.Parser.ParseFrom(Field<ByteString>(terminal, "payload"));
    }
    throw new InvalidOperationException("Invoke: Missing Unary Operation Response");
  }

  internal async Task<IReadOnlyList<Sample>> TickAsync(string stream, string scope, Timestamp at)
  {
    using var call = calls.AsyncDuplexStreamingCall(Method(MethodType.DuplexStreaming, "Samples"), null, new CallOptions());
    var selection = Frame("SamplesSelection", ("stream", stream), ("scope", scope));
    await call.RequestStream.WriteAsync(Frame("InstanceSamplesRequest", ("open", selection)).ToByteString());
    var tick = Frame("Tick", ("at", at));
    await call.RequestStream.WriteAsync(Frame("InstanceSamplesRequest", ("tick", tick)).ToByteString());
    if (!await call.ResponseStream.MoveNext())
      throw new InvalidOperationException("Tick Observer: Missing Sample Response");
    var frame = Descriptor("InstanceSamplesResponse").Parser.ParseFrom(call.ResponseStream.Current);
    var samples = Field<IEnumerable>(frame, "samples").Cast<IMessage>()
        .Select(sample => new Sample(Field<Timestamp>(sample, "at"), Field<ByteString>(sample, "row"))).ToList();
    await call.RequestStream.CompleteAsync();
    while (await call.ResponseStream.MoveNext())
    {
    }
    return samples;
  }

  internal async Task StopAsync()
  {
    using var call = calls.AsyncUnaryCall(Method(MethodType.Unary, "Stop"), null, new CallOptions(),
        Frame("InstanceStopRequest").ToByteString());
    await call.ResponseAsync;
  }
}
