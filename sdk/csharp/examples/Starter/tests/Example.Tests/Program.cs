/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

using Example.Plugin;

/* === Entry Point === */

var plugin = new ExamplePlugin();
var description = plugin.Describe();
var suite = description.Suites.Single();
var template = suite.Templates.Single();
if (suite.Identifier != "example.capture" || template.Identifier != "example.capture.confirmation"
  || template.Steps.Single() != "example.capture.confirmation.confirm"
  || template.Criteria.Single() != "example.capture.confirmation.confirmed"
  || plugin.Records.Single().Identifier != "confirmation"
  || !description.Operations.Any(operation => operation.Identifier == "example.echo"))
{
  throw new InvalidOperationException("Describe Plugin: Restore the Authored Capture Suite and Echo Operation");
}
Console.WriteLine("Plugin Description Passed");
