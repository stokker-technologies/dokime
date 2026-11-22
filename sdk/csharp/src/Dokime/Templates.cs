/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

namespace Dokime.Templates;

public abstract class TestTemplate
{
  public virtual string Identifier => Identifiers.Build(GetType().Name, type: true);
  public virtual string Label => AuthorNames.Label(GetType().Name, "TestTemplate");
  public virtual IReadOnlyList<Parameter> Parameters => [];
  public virtual IReadOnlyList<Document> Documents => [];
  public virtual TestCase Materialize(TestCase testCase) => testCase;
  public virtual Task InitializeAsync(Context context, CancellationToken cancellationToken = default) => Task.CompletedTask;
  public virtual Task CleanupAsync(Context context, CancellationToken cancellationToken = default) => Task.CompletedTask;
}
