// SPDX-License-Identifier: Apache-2.0

using Google.Protobuf.WellKnownTypes;

namespace Dokime.Operations;

/// <summary>Typed Settings From One Host-Validated Activation Snapshot.</summary>
public abstract class Settings
{
  private Harmos.SettingValues? values;
  public abstract IReadOnlyList<Harmos.SettingDeclaration> Declarations
  {
    get;
  }

  internal void Load(Harmos.SettingValues snapshot) => values = snapshot.Clone();

  protected T Read<T>(string name)
  {
    if (values is null || !values.Values.TryGetValue(name, out var value))
      throw new ArgumentException($"Read Settings: Configure {name} and Activate the Plugin");
    object result = value.ValueCase switch
    {
      Harmos.SettingValue.ValueOneofCase.Flag => value.Flag,
      Harmos.SettingValue.ValueOneofCase.Integer => value.Integer,
      Harmos.SettingValue.ValueOneofCase.Number => value.Number,
      Harmos.SettingValue.ValueOneofCase.Text => value.Text,
      Harmos.SettingValue.ValueOneofCase.Duration => value.Duration.ToTimeSpan(),
      _ => throw new ArgumentException($"Read Settings: Save a Typed Value for {name}")
    };
    return result is T typed ? typed : throw new ArgumentException($"Read Settings: Save the Declared Type for {name}");
  }
}
