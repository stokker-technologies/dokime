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

var description = new ExamplePlugin().Describe();
var suite = description.Suites.Single();
var template = suite.Templates.Single();
if (suite.Identifier != "capture" || template.Identifier != "capture.confirmation"
  || template.Steps.Single() != "capture.confirmation.confirm"
  || template.Criteria.Single() != "capture.confirmation.confirmed"
  || template.Records.Single() != "capture.confirmation.confirmation"
  || !description.Operations.Any(operation => operation.Identifier == "example.echo"))
{
  throw new InvalidOperationException("Describe Plugin: Restore the Authored Capture Suite and Echo Operation");
}
Console.WriteLine("Plugin Description Passed");
