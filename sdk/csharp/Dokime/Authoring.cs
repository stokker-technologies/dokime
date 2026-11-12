// SPDX-License-Identifier: Apache-2.0

using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Harmos.Sdk;
using Dokime.V1;

namespace Dokime;

public sealed record Step(string Identifier, string Label, Func<Context, Task> Run, uint Retries = 0, bool Repeatable = false);
public sealed record Criterion(string Identifier, string Label, Func<Context, Task<bool>> Evaluate);
public sealed record Record<T>(string Identifier, string Label) where T : IMessage<T>, new();

public interface IStream
{
  string Identifier { get; }
  internal void Register(Harmos.Sdk.Plugin plugin, Func<Harmos.Sdk.Context, Context> context);
}

/// <summary>A device-driven stream; an Observer supplies host-ticked samples instead.</summary>
public sealed class Stream<T>(string identifier, Func<Context, Publisher<T>, Task>? produce = null) : IStream where T : IMessage<T>, new()
{
  public string Identifier { get; } = identifier;
  internal Observer<T>? Observer { get; set; }
  void IStream.Register(Harmos.Sdk.Plugin plugin, Func<Harmos.Sdk.Context, Context> context)
  {
    if (Observer is { } observer)
      plugin.Observer<T>(Identifier, observer.Interval, raw => observer.Read(context(raw)), intervalSetting: observer.IntervalSetting);
    else if (produce is not null)
      plugin.Stream<T>(Identifier, (raw, publisher) => produce(context(raw), publisher));
    else
      throw new ArgumentException($"Stream {Identifier} needs a producer or observer");
  }
}

public sealed class Observer<T> where T : IMessage<T>, new()
{
  public Observer(Stream<T> stream, Duration? interval, Func<Context, Task<T>> read, string intervalSetting = "")
  {
    if (stream.Observer is not null)
      throw new ArgumentException($"Duplicate observer for {stream.Identifier}");
    Stream = stream;
    Interval = interval?.Clone();
    Read = read;
    IntervalSetting = intervalSetting;
    stream.Observer = this;
  }
  public Stream<T> Stream { get; }
  public Duration? Interval { get; }
  public string IntervalSetting { get; }
  public Func<Context, Task<T>> Read { get; }
}

public sealed class Template(string identifier, string label)
{
  public TestCaseTemplate Definition { get; } = new() { Metadata = new Metadata { Identifier = identifier, Label = label } };
  internal List<Step> Steps { get; } = [];
  internal List<Criterion> Criteria { get; } = [];
  internal List<IStream> Streams { get; } = [];
  internal Func<Context, Task>? InitializeBody { get; private set; }
  internal Func<Context, Task>? CleanupBody { get; private set; }
  internal Func<Context, Task<IAsyncDisposable>>? CreateResource { get; private set; }
  internal Func<TestCase, IReadOnlyDictionary<string, V1.Value>, TestCase>? MaterializeBody { get; private set; }
  public Template Step(Step step) { Steps.Add(step); return this; }
  public Template Criterion(Criterion criterion) { Criteria.Add(criterion); return this; }
  public Template Stream(IStream stream) { Streams.Add(stream); return this; }
  public Template Observer<T>(Observer<T> observer) where T : IMessage<T>, new() => Stream(observer.Stream);
  public Template Initialize(Func<Context, Task> initialize) { InitializeBody = initialize; return this; }
  public Template Cleanup(Func<Context, Task> cleanup) { CleanupBody = cleanup; return this; }
  public Template Resource(Func<Context, Task<IAsyncDisposable>> create) { CreateResource = create; return this; }
  public Template Materialize(Func<TestCase, IReadOnlyDictionary<string, V1.Value>, TestCase> materialize) { MaterializeBody = materialize; return this; }
  public Template Parameter(Parameter parameter) { Definition.Parameters.Add(parameter.Clone()); return this; }
  public Template Document(Document document) { Definition.Documents.Add(document.Clone()); return this; }
}

public sealed class Suite(string identifier, string label, params Template[] templates)
{
  public Metadata Metadata { get; } = new() { Identifier = identifier, Label = label };
  public IReadOnlyList<Template> Templates { get; } = templates;
}
