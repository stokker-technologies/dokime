// SPDX-License-Identifier: Apache-2.0

namespace Dokime.Operations;

/// <summary>A Lazily Opened Resource Shared by One Plugin Activation.</summary>
public abstract class Resource
{
  public virtual Task OpenAsync(Context context) => Task.CompletedTask;
  public virtual Task CloseAsync() => Task.CompletedTask;
}
