// SPDX-License-Identifier: Apache-2.0

namespace Dokime.Templates;

public sealed record SampleWindow(DateTimeOffset Start, TimeSpan Period, ulong Count);

public sealed class Publisher<T>
{
  private readonly Func<T, DateTimeOffset?, ValueTask> publish;
  internal Publisher(Func<T, DateTimeOffset?, ValueTask> publish) => this.publish = publish;
  public ValueTask PublishAsync(T value, DateTimeOffset? at = null) => publish(value, at);
}
