// SPDX-License-Identifier: Apache-2.0

using System.Numerics;
using System.Globalization;
using Google.Protobuf.WellKnownTypes;

namespace Dokime;

public sealed record Rational(BigInteger Numerator, BigInteger Denominator)
{
  internal ExactNumber ToWire()
  {
    if (Denominator.IsZero)
      throw new ArgumentException("Encode Rational: Supply a Nonzero Denominator");
    var divisor = BigInteger.GreatestCommonDivisor(Numerator, Denominator);
    return new ExactNumber
    {
      Negative = Numerator.Sign * Denominator.Sign < 0,
      Numerator = BigInteger.Abs(Numerator / divisor).ToString(CultureInfo.InvariantCulture),
      Denominator = BigInteger.Abs(Denominator / divisor).ToString(CultureInfo.InvariantCulture)
    };
  }
}
public sealed record Statistics(ulong Count, object? Min, object? Max, object? Mean, double Stddev, object? Median,
  object? P05, object? P95, object? P99, object? First, object? Last, object? Sum)
{
  internal static Statistics FromWire(Clients.Statistics value) => new(value.Count, Scalars.Optional(value.Min), Scalars.Optional(value.Max),
    Scalars.Optional(value.Mean), value.Stddev, Scalars.Optional(value.Median), Scalars.Optional(value.P05), Scalars.Optional(value.P95),
    Scalars.Optional(value.P99), Scalars.Optional(value.First), Scalars.Optional(value.Last), Scalars.Optional(value.Sum));
}

public sealed record RecordRow<T>(string Identifier, ulong Sequence, long TimestampNs, long ReceivedAtNs, string RecordType,
  uint SchemaVersion, string Source, T Value);
public sealed record SampleRow<T>(string Identifier, ulong Sequence, long TimestampNs, long ReceivedAtNs, string Stream,
  uint SchemaVersion, string Source, T Value, IReadOnlyDictionary<string, object?> Values);

internal static class EvidenceTime
{
  internal static long Nanoseconds(Timestamp value) => checked(value.Seconds * 1_000_000_000 + value.Nanos);
}
