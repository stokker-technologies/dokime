/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

using System.Text;
using System.Text.RegularExpressions;

namespace Dokime;

public sealed record ConnectionSettings
{
  public string? Remote
  {
    get; init;
  }
  public string? PluginAddress
  {
    get; init;
  }
  public TimeSpan ConnectTimeout { get; init; } = TimeSpan.FromSeconds(30);
}

internal static class Configuration
{
  internal static Dictionary<string, string> Load()
  {
    var path = Path.Combine(Environment.CurrentDirectory, ".env");
    byte[] bytes = [];
    try
    {
      bytes = System.IO.File.ReadAllBytes(path);
      return Parse(new UTF8Encoding(false, true).GetString(bytes), path);
    }
    catch (FileNotFoundException)
    {
      return [];
    }
    catch (DecoderFallbackException error)
    {
      var number = bytes.Take(Math.Max(0, error.Index)).Count(value => value == (byte)'\n') + 1;
      throw new ArgumentException($"Read {path}:{number}: Save the File as UTF-8", error);
    }
  }

  internal static string Resolve(string key, string? explicitValue, IReadOnlyDictionary<string, string> values)
  {
    var fallback = key == "DOKIME_PLUGIN_ADDRESS" ? "127.0.0.1:0" : "";
    var value = explicitValue ?? Environment.GetEnvironmentVariable(key) ?? values.GetValueOrDefault(key, fallback);
    if (value.Length == 0)
      throw new ArgumentException($"Configure Connection: Set {key} to a Nonempty Address");
    return value;
  }

  internal static Dictionary<string, string> Parse(string content, string path)
  {
    Dictionary<string, string> values = [];
    var lines = (content.StartsWith('\uFEFF') ? content[1..] : content).Split('\n');
    for (var number = 0; number < lines.Length; number++)
    {
      var raw = number < lines.Length - 1 && lines[number].EndsWith('\r') ? lines[number][..^1] : lines[number];
      var line = raw.Trim(' ', '\t');
      if (line.Contains('\0') || line.Contains('\r'))
        throw Malformed(path, number + 1);
      if (line.Length == 0 || line.StartsWith('#'))
        continue;
      var match = Regex.Match(line, @"^([A-Za-z_][A-Za-z0-9_]*)[ \t]*=[ \t]*(.*)$");
      if (!match.Success || values.ContainsKey(match.Groups[1].Value))
        throw Malformed(path, number + 1);
      var key = match.Groups[1].Value;
      var value = match.Groups[2].Value;
      if (!value.StartsWith('"') && !value.StartsWith('\''))
      {
        values.Add(key, Regex.Split(value, @"(?:^[#]|[ \t]+[#])", RegexOptions.None)[0].Trim(' ', '\t'));
        continue;
      }
      var quote = value[0];
      var decoded = new StringBuilder();
      var index = 1;
      while (index < value.Length && value[index] != quote)
      {
        var character = value[index];
        if (character == '\\' && quote == '"')
        {
          index++;
          if (index == value.Length)
            throw Malformed(path, number + 1);
          character = value[index] switch
          {
            '\\' => '\\',
            '"' => '"',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            _ => throw Malformed(path, number + 1)
          };
        }
        decoded.Append(character);
        index++;
      }
      if (index == value.Length || !Regex.IsMatch(value[(index + 1)..], @"^[ \t]*(?:#.*)?$"))
        throw Malformed(path, number + 1);
      values.Add(key, decoded.ToString());
    }
    return values;
  }

  private static ArgumentException Malformed(string path, int number) =>
    new($"Read {path}:{number}: Use One Unique KEY=VALUE Assignment with Balanced Quotes and Supported Escapes");
}
