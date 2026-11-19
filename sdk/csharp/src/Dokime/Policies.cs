// SPDX-License-Identifier: Apache-2.0

using Google.Protobuf.WellKnownTypes;

namespace Dokime.Operations;

public sealed record RetryPolicy(uint TotalAttempts, TimeSpan Initial, TimeSpan? Maximum = null, bool DeterministicJitter = false)
{
  internal Harmos.RetryPolicy ToWire()
  {
    if (TotalAttempts == 0 || Initial <= TimeSpan.Zero || Maximum < Initial)
      throw new ArgumentException("Declare Retry Policy: Supply Positive Attempts and Backoff with Maximum at Least Initial");
    return new Harmos.RetryPolicy { TotalAttempts = TotalAttempts, Initial = Duration.FromTimeSpan(Initial), Maximum = Maximum is null ? null : Duration.FromTimeSpan(Maximum.Value), DeterministicJitter = DeterministicJitter };
  }
}

public sealed record OperationPolicy(TimeSpan? Timeout = null, RetryPolicy? Retries = null)
{
  internal Harmos.OperationPolicy ToWire()
  {
    if (Timeout <= TimeSpan.Zero)
      throw new ArgumentException("Declare Operation Policy: Supply a Positive Timeout");
    return new Harmos.OperationPolicy { Timeout = Timeout is null ? null : Duration.FromTimeSpan(Timeout.Value), Retries = Retries?.ToWire() };
  }
}

public abstract class PolicyAttribute : Attribute
{
  public double TimeoutSeconds
  {
    get; set;
  }
  public uint TotalAttempts
  {
    get; set;
  }
  public double InitialBackoffSeconds
  {
    get; set;
  }
  public double MaximumBackoffSeconds
  {
    get; set;
  }
  public bool ReplaySafe
  {
    get; set;
  }
  internal OperationPolicy? Policy()
  {
    if (!double.IsFinite(TimeoutSeconds) || TimeoutSeconds < 0 || !double.IsFinite(InitialBackoffSeconds)
      || !double.IsFinite(MaximumBackoffSeconds) || TotalAttempts == 0 && (InitialBackoffSeconds != 0 || MaximumBackoffSeconds != 0))
      throw new ArgumentException("Declare Operation Policy: Supply Finite Positive Durations");
    var retries = TotalAttempts == 0 ? null : new RetryPolicy(TotalAttempts, TimeSpan.FromSeconds(InitialBackoffSeconds),
      MaximumBackoffSeconds == 0 ? null : TimeSpan.FromSeconds(MaximumBackoffSeconds));
    var result = new OperationPolicy(TimeoutSeconds == 0 ? null : TimeSpan.FromSeconds(TimeoutSeconds), retries);
    _ = result.ToWire();
    return TimeoutSeconds == 0 && retries is null ? null : result;
  }
}
