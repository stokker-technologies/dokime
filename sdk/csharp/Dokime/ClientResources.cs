// @generated from the shared resource table; do not edit.
namespace Dokime;

public sealed partial class Client
{
  public Sdk.AccessResource Access => new(Calls);
  public Sdk.CatalogEntriesResource CatalogEntries => new(Calls);
  public Sdk.ExecutionsResource Executions => new(Calls);
  public Sdk.InvocationsResource Invocations => new(Calls);
  public Sdk.PluginsResource Plugins => new(Calls);
  public Sdk.StatusResource Status => new(Calls);
  public Sdk.StorageResource Storage => new(Calls);
  public Sdk.TimeResource Time => new(Calls);
}
