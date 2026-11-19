/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

using Dokime;

namespace Dokime.Templates;

public static class Parameters
{
  private static Parameter Create(string identifier, string label, Dokime.ValueType type) => new()
  {
    Metadata = new Metadata { Identifier = identifier, Label = label },
    Type = type
  };
  public static Parameter Number(string identifier, string label) => Create(identifier, label, Dokime.ValueType.Number);
  public static Parameter Count(string identifier, string label) => Create(identifier, label, Dokime.ValueType.UnsignedInteger);
  public static Parameter Flag(string identifier, string label) => Create(identifier, label, Dokime.ValueType.Boolean);
  public static Parameter Sweep(string identifier, string label, IEnumerable<double> values)
  {
    var result = Number(identifier, label);
    result.Expression = new ParameterExpression { Array = new ParameterExpressions() };
    foreach (var value in values)
      result.Expression.Array.Values.Add(new ParameterExpression { Literal = new Value { Number = value } });
    result.Expansion = new ParameterExpansion { Cartesian = true };
    return result;
  }
  public static Parameter Numbers(string identifier, string label, IEnumerable<double> values)
  {
    var result = Create(identifier, label, Dokime.ValueType.List);
    result.ElementType = Dokime.ValueType.Number;
    var list = new ValueList();
    foreach (var value in values)
      list.Values.Add(new Value { Number = value });
    result.Expression = new ParameterExpression { Literal = new Value { List = list } };
    return result;
  }
  public static Parameter WithUnit(this Parameter parameter, string unit)
  {
    parameter.Unit = unit;
    return parameter;
  }
  public static Parameter WithDefault(this Parameter parameter, double value)
  {
    parameter.Expression = new ParameterExpression { Literal = new Value { Number = value } };
    return parameter;
  }
  public static Parameter WithDefault(this Parameter parameter, ulong value)
  {
    parameter.Expression = new ParameterExpression { Literal = new Value { UnsignedInteger = value } };
    return parameter;
  }
  public static Parameter WithDefault(this Parameter parameter, bool value)
  {
    parameter.Expression = new ParameterExpression { Literal = new Value { Boolean = value } };
    return parameter;
  }
  public static Parameter Range(this Parameter parameter, double? minimum = null, double? maximum = null)
  {
    if (minimum.HasValue)
      parameter.Min = minimum.Value;
    else
      parameter.ClearMin();
    if (maximum.HasValue)
      parameter.Max = maximum.Value;
    else
      parameter.ClearMax();
    return parameter;
  }
  public static Parameter Items(this Parameter parameter, uint? minimum = null, uint? maximum = null)
  {
    if (minimum.HasValue)
      parameter.MinItems = minimum.Value;
    else
      parameter.ClearMinItems();
    if (maximum.HasValue)
      parameter.MaxItems = maximum.Value;
    else
      parameter.ClearMaxItems();
    return parameter;
  }
}

public static class Cards
{
  public static Card Plot(Plot plot) => new() { Metadata = plot.Metadata?.Clone(), Plot = plot.Clone() };
  public static Card Records(string identifier) => new() { Metadata = new Metadata { Identifier = identifier, Label = "Records" }, Evidence = EvidenceCard.Records };
  public static Card Columns(this Card card, uint columns)
  {
    card.Layout ??= new CardLayout();
    card.Layout.Columns = columns;
    return card;
  }
}
