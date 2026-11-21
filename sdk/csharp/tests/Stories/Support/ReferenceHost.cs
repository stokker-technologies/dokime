// SPDX-License-Identifier: Apache-2.0

using System.Diagnostics;
using System.Net;
using Google.Protobuf;
using Google.Protobuf.Reflection;
using Grpc.Core;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Server.Kestrel.Core;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace Stories.Support;

internal static class ReferenceHost
{
  internal static string Artifact(string name)
  {
    var path = Environment.GetEnvironmentVariable(name);
    if (string.IsNullOrEmpty(path) || !File.Exists(path))
      throw new InvalidOperationException($"Missing Build Output: {name}; Run mise -C sdk/csharp run build:reference and build");
    return path;
  }

  internal static async Task<(int ExitCode, string Evidence)> RunAsync(string host, string plugin)
  {
    var start = new ProcessStartInfo(host) { RedirectStandardOutput = true, RedirectStandardError = true };
    start.ArgumentList.Add("--plugin");
    start.ArgumentList.Add(plugin);
    // The Reference Host Owns Startup, Per-Case and Shutdown Transport Hang Guards.
    using var process = Process.Start(start) ?? throw new InvalidOperationException("Reference Host Did Not Start");
    var stdout = process.StandardOutput.ReadToEndAsync();
    var stderr = process.StandardError.ReadToEndAsync();
    await process.WaitForExitAsync();
    return (process.ExitCode, await stdout + await stderr);
  }

  internal static async Task<(int ExitCode, string Evidence, bool StillServing)> RunRemoteAsync(string host, string plugin)
  {
    await using var launch = await LaunchHost.StartAsync();
    var start = launch.Command(plugin);
    start.ArgumentList.Add("--remote");
    using var server = Process.Start(start) ?? throw new InvalidOperationException("Remote Plugin Did Not Start");
    var diagnostics = server.StandardError.ReadToEndAsync();
    var output = server.StandardOutput.ReadToEndAsync();
    (int ExitCode, string Evidence, bool StillServing) result;
    try
    {
      // Guard Only Transport Setup; Scenario Time Still Comes Exclusively from Host Ticks.
      using var startup = new CancellationTokenSource(TimeSpan.FromSeconds(30));
      var readiness = await launch.Address.WaitAsync(startup.Token);
      var request = new ProcessStartInfo(host) { RedirectStandardOutput = true, RedirectStandardError = true };
      request.ArgumentList.Add("--address");
      request.ArgumentList.Add(readiness);
      using var process = Process.Start(request) ?? throw new InvalidOperationException("Reference Host Did Not Start");
      var stdout = process.StandardOutput.ReadToEndAsync();
      var stderr = process.StandardError.ReadToEndAsync();
      await process.WaitForExitAsync();
      result = (process.ExitCode, await stdout + await stderr, !server.HasExited);
    }
    finally
    {
      if (!server.HasExited)
        server.Kill(entireProcessTree: true);
      await server.WaitForExitAsync();
      await diagnostics;
      await output;
    }
    return (result.ExitCode, result.Evidence + await diagnostics, result.StillServing);
  }
}

internal sealed class LaunchHost(WebApplication app, LaunchHost.Announcements service) : IAsyncDisposable
{
  internal Task<string> Address => service.Address.Task;

  internal ProcessStartInfo Command(string plugin)
  {
    var start = new ProcessStartInfo(plugin) { RedirectStandardOutput = true, RedirectStandardError = true };
    start.Environment["HARMOS_HOST_ENDPOINT"] = new Uri(app.Urls.Single()).Authority;
    start.Environment["HARMOS_LAUNCH_TOKEN"] = service.Token;
    return start;
  }

  internal static async Task<LaunchHost> StartAsync()
  {
    var service = new Announcements();
    var builder = WebApplication.CreateSlimBuilder();
    builder.Logging.ClearProviders();
    builder.WebHost.ConfigureKestrel(options => options.Listen(IPAddress.Loopback, 0, listener => listener.Protocols = HttpProtocols.Http2));
    builder.Services.AddGrpc();
    builder.Services.AddSingleton(service);
    var app = builder.Build();
    try
    {
      app.MapGrpcService<Announcements>();
      await app.StartAsync();
      return new LaunchHost(app, service);
    }
    catch
    {
      await app.DisposeAsync();
      throw;
    }
  }

  public async ValueTask DisposeAsync()
  {
    try
    {
      await app.StopAsync();
    }
    finally { await app.DisposeAsync(); }
  }

  // Use the Same Generated Carrier Descriptors as Wire; SDK Transport Types Stay Internal.
  [BindServiceMethod(typeof(Announcements), nameof(BindService))]
  internal class Announcements
  {
    private static readonly FileDescriptor Descriptor = (FileDescriptor)typeof(Harmos.Plugins.Plugin).Assembly
        .GetType("Harmos.Internal.Plugin.LaunchesReflection", throwOnError: true)!
        .GetProperty("Descriptor")!.GetValue(null)!;
    internal string Token { get; } = Guid.NewGuid().ToString("N");
    internal TaskCompletionSource<string> Address { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);

    public static void BindService(ServiceBinderBase binder, Announcements service)
    {
      var rpc = Descriptor.Services.Single().Methods.Single();
      var method = new Method<IMessage, IMessage>(MethodType.Unary, rpc.Service.FullName, rpc.Name,
          Marshallers.Create<IMessage>(message => message.ToByteArray(), bytes => rpc.InputType.Parser.ParseFrom(bytes)),
          Marshallers.Create<IMessage>(message => message.ToByteArray(), bytes => rpc.OutputType.Parser.ParseFrom(bytes)));
      binder.AddMethod(method, service is null ? null : service.Announce);
    }

    public virtual Task<IMessage> Announce(IMessage request, ServerCallContext context)
    {
      if ((string)request.Descriptor.FindFieldByName("token").Accessor.GetValue(request) != Token)
        throw new RpcException(new Status(StatusCode.Unauthenticated, "Announce Plugin: Use the Assigned Launch Token"));
      if (!Address.TrySetResult((string)request.Descriptor.FindFieldByName("address").Accessor.GetValue(request)))
        throw new RpcException(new Status(StatusCode.AlreadyExists, "Announce Plugin: Start a New Launch Reservation"));
      return Task.FromResult(Descriptor.Services.Single().Methods.Single().OutputType.Parser.ParseFrom(ByteString.Empty));
    }
  }
}
