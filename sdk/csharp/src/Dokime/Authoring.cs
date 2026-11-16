// SPDX-License-Identifier: Apache-2.0

using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Harmos.Plugins;
using Harmos;
using Dokime;
using Value = Dokime.Value;
using TransportContext = Harmos.Plugins.Context;
using TransportPlugin = Harmos.Plugins.Plugin;
using SampleWindow = Harmos.Plugins.Window;

namespace Dokime.Plugins;

public sealed record Step(string Identifier, string Label, Func<Context, Task> Run,
    OperationPolicy? Policy = null, bool ReplaySafe = false, bool Repeatable = false);
public sealed record Criterion(string Identifier, string Label, Func<Context, Task<bool>> Evaluate, OperationPolicy? Policy = null);
public sealed record Record<T>(string Identifier, string Label) where T : IMessage<T>, new();

public interface IStream
{
  string Identifier
  {
    get;
  }
  internal void Register(TransportPlugin plugin, Func<TransportContext, Context> context);
}

/// <summary>Registers One Observer Form for Its Stream.</summary>
internal interface ISampler
{
  void Register(TransportPlugin plugin, string stream, Func<TransportContext, Context> context);
}

/// <summary>A device-driven stream; an Observer supplies samples in host-granted windows instead.</summary>
public sealed class Stream<T>(string identifier, Func<Context, Publisher<T>, Task>? produce = null) : IStream where T : IMessage<T>, new()
{
  public string Identifier
  {
    get;
  } = identifier;
  internal ISampler? Observer
  {
    get;
    set;
  }
  void IStream.Register(TransportPlugin plugin, Func<TransportContext, Context> context)
  {
    if (Observer is { } observer)
      observer.Register(plugin, Identifier, context);
    else if (produce is not null)
      plugin.Stream<T>(Identifier, (raw, publisher) => produce(context(raw), publisher));
    else
      throw new ArgumentException($"Stream {Identifier} needs a producer or observer");
  }
}

/// <summary>Samples One Row per Call; Each Host Window Calls It Once per Sample, in Order.</summary>
/// <remarks><c>windowSetting</c> names a plugin Duration setting that replaces the host's window length.</remarks>
public sealed class Observer<T> : ISampler where T : IMessage<T>, new()
{
  public Observer(Stream<T> stream, Duration? interval, Func<Context, Task<T>> read, string intervalSetting = "", string windowSetting = "")
  {
    if (stream.Observer is not null)
      throw new ArgumentException($"Duplicate observer for {stream.Identifier}");
    Stream = stream;
    Interval = interval?.Clone();
    Read = read;
    IntervalSetting = intervalSetting;
    WindowSetting = windowSetting;
    stream.Observer = this;
  }
  void ISampler.Register(TransportPlugin plugin, string stream, Func<TransportContext, Context> context) =>
      plugin.Observer<T>(stream, Interval, raw => Read(context(raw)), intervalSetting: IntervalSetting, windowSetting: WindowSetting);
  public string WindowSetting
  {
    get;
  }
  public Stream<T> Stream
  {
    get;
  }
  public Duration? Interval
  {
    get;
  }
  public string IntervalSetting
  {
    get;
  }
  public Func<Context, Task<T>> Read
  {
    get;
  }
}

/// <summary>Answers a Whole Host Window per Call With Columns That Repeat Every Field of the Row.</summary>
/// <remarks>Sample <c>i</c> is due at <c>window.Start + i * window.Period</c>; every column holds <c>window.Count</c> values.</remarks>
public sealed class WindowObserver<T, TColumns> : ISampler
    where T : IMessage<T>, new()
    where TColumns : IMessage<TColumns>, new()
{
  public WindowObserver(Stream<T> stream, Duration? interval, Func<Context, SampleWindow, Task<TColumns>> read,
      string intervalSetting = "", string windowSetting = "")
  {
    if (stream.Observer is not null)
      throw new ArgumentException($"Duplicate observer for {stream.Identifier}");
    Stream = stream;
    Interval = interval?.Clone();
    Read = read;
    IntervalSetting = intervalSetting;
    WindowSetting = windowSetting;
    stream.Observer = this;
  }
  void ISampler.Register(TransportPlugin plugin, string stream, Func<TransportContext, Context> context) =>
      plugin.WindowObserver<T, TColumns>(stream, Interval, (raw, window) => Read(context(raw), window),
          intervalSetting: IntervalSetting, windowSetting: WindowSetting);
  public Stream<T> Stream
  {
    get;
  }
  public Duration? Interval
  {
    get;
  }
  public string IntervalSetting
  {
    get;
  }
  public string WindowSetting
  {
    get;
  }
  public Func<Context, SampleWindow, Task<TColumns>> Read
  {
    get;
  }
}

public sealed class Template(string identifier, string label)
{
  public TestCaseTemplate Definition
  {
    get;
  } = new()
  {
    Metadata = new Metadata { Identifier = identifier, Label = label }
  };
  internal List<Step> Steps
  {
    get;
  } = [];
  internal List<Criterion> Criteria
  {
    get;
  } = [];
  internal List<IStream> Streams
  {
    get;
  } = [];
  internal Func<Context, Task>? InitializeBody
  {
    get;
    private set;
  }
  internal Func<Context, Task>? CleanupBody
  {
    get;
    private set;
  }
  internal Func<Context, Task<IAsyncDisposable>>? CreateResource
  {
    get;
    private set;
  }
  internal Func<TestCase, IReadOnlyDictionary<string, Value>, TestCase>? MaterializeBody
  {
    get;
    private set;
  }
  internal OperationPolicy? MaterializePolicy
  {
    get;
    private set;
  }
  internal OperationPolicy? InitializePolicy
  {
    get;
    private set;
  }
  internal OperationPolicy? CleanupPolicy
  {
    get;
    private set;
  }
  public Template Step(Step step)
  {
    Steps.Add(step);
    return this;
  }
  public Template Criterion(Criterion criterion)
  {
    Criteria.Add(criterion);
    return this;
  }
  public Template Stream(IStream stream)
  {
    Streams.Add(stream);
    return this;
  }
  public Template Observer<T>(Observer<T> observer) where T : IMessage<T>, new() => Stream(observer.Stream);
  public Template Observer<T, TColumns>(WindowObserver<T, TColumns> observer)
      where T : IMessage<T>, new()
      where TColumns : IMessage<TColumns>, new() => Stream(observer.Stream);
  public Template Initialize(Func<Context, Task> initialize, OperationPolicy? policy = null)
  {
    InitializeBody = initialize;
    InitializePolicy = policy?.Clone();
    return this;
  }
  public Template Cleanup(Func<Context, Task> cleanup, OperationPolicy? policy = null)
  {
    CleanupBody = cleanup;
    CleanupPolicy = policy?.Clone();
    return this;
  }
  public Template Resource(Func<Context, Task<IAsyncDisposable>> create)
  {
    CreateResource = create;
    return this;
  }
  public Template Materialize(Func<TestCase, IReadOnlyDictionary<string, Value>, TestCase> materialize, OperationPolicy? policy = null)
  {
    MaterializeBody = materialize;
    MaterializePolicy = policy?.Clone();
    return this;
  }
  public Template Parameter(Parameter parameter)
  {
    Definition.Parameters.Add(parameter.Clone());
    return this;
  }
  public Template Document(Document document)
  {
    Definition.Documents.Add(document.Clone());
    return this;
  }
}

public sealed class Suite(string identifier, string label, params Template[] templates)
{
  public Metadata Metadata
  {
    get;
  } = new()
  {
    Identifier = identifier,
    Label = label
  };
  public IReadOnlyList<Template> Templates
  {
    get;
  } = templates;
}
