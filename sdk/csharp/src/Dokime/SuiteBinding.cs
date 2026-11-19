// SPDX-License-Identifier: Apache-2.0

using Dokime.Templates;

namespace Dokime.Plugins;

internal sealed record SuiteBinding(Metadata Metadata, IReadOnlyList<Template> Templates);
