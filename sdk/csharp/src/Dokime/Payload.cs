// SPDX-License-Identifier: Apache-2.0

using Google.Protobuf;
using Google.Protobuf.Reflection;
using Google.Protobuf.WellKnownTypes;

namespace Dokime.Plugins;

internal static class Payload<T>
{
  internal static MessageDescriptor Descriptor => Empty().Descriptor;

  private static IMessage Empty()
  {
    var type = typeof(T);
    if (type == typeof(bool))
      return new BoolValue();
    if (type == typeof(string))
      return new StringValue();
    if (type == typeof(byte[]))
      return new BytesValue();
    if (type == typeof(int) || type == typeof(long))
      return new Int64Value();
    if (type == typeof(float) || type == typeof(double))
      return new DoubleValue();
    if (typeof(IMessage).IsAssignableFrom(type) && Activator.CreateInstance(type) is IMessage message)
      return message;
    throw new ArgumentException("Declare Payload: Supply bool, string, int, long, float, double, byte[] or a Protobuf Message");
  }

  internal static IMessage Pack(T value)
  {
    return value switch
    {
      bool flag => new BoolValue { Value = flag },
      string text => new StringValue { Value = text },
      byte[] bytes => new BytesValue { Value = ByteString.CopyFrom(bytes) },
      int number => new Int64Value { Value = number },
      long number => new Int64Value { Value = number },
      float number => new DoubleValue { Value = number },
      double number => new DoubleValue { Value = number },
      IMessage message => message,
      _ => throw new ArgumentException("Encode Payload: Supply a Declared Plain Value or Protobuf Message")
    };
  }

  internal static T Unpack(ByteString bytes)
  {
    var message = Descriptor.Parser.ParseFrom(bytes);
    object value = message switch
    {
      BoolValue flag when typeof(T) == typeof(bool) => flag.Value,
      StringValue text when typeof(T) == typeof(string) => text.Value,
      BytesValue binary when typeof(T) == typeof(byte[]) => binary.Value.ToByteArray(),
      Int64Value number when typeof(T) == typeof(int) => checked((int)number.Value),
      Int64Value number when typeof(T) == typeof(long) => number.Value,
      DoubleValue number when typeof(T) == typeof(float) => (float)number.Value,
      DoubleValue number when typeof(T) == typeof(double) => number.Value,
      _ => message
    };
    return (T)value;
  }

  internal static void Register(Harmos.Plugins.Plugin plugin)
  {
    typeof(Harmos.Plugins.Plugin).GetMethod(nameof(Harmos.Plugins.Plugin.Message))!
        .MakeGenericMethod(Descriptor.ClrType).Invoke(plugin, null);
  }
}
