## ADDED Requirements

### Requirement: Starters Can Be Created From a Checkout

`dokime new` SHALL accept `--from <checkout>`. With it, the starter files SHALL
come from that checkout's starter for the selected language and the created
project's SDK dependency SHALL be a local path to that checkout's SDK. Nothing
SHALL be downloaded and no published SDK SHALL be used as a fallback.

#### Scenario: Python From a Checkout

- **WHEN** a Python starter is created with `--from` naming a checkout
- **THEN** the project depends on that checkout's SDK by path and installs without network access to a release

#### Scenario: Checkout Lacks the Starter

- **WHEN** `--from` names a folder without the selected language's starter
- **THEN** the command fails and names the missing path

### Requirement: Environment Selects the Same Source

`DOKIME_NEW_FROM` SHALL select the checkout when the argument is absent, and the
argument SHALL take precedence over it.

#### Scenario: Argument Over Environment

- **WHEN** both the variable and the argument are set to different checkouts
- **THEN** the argument's checkout is used

### Requirement: Default Creation Is Unchanged

Without `--from` and without the variable, `dokime new` SHALL create the starter
embedded in the tool with the SDK taken from the published channel matching the
tool's build.

#### Scenario: No Source Given

- **WHEN** a starter is created with no source
- **THEN** it is the embedded starter and its SDK dependency names the published channel

### Requirement: Starter Stories Use the Checkout Under Test

Every automated story that creates a starter SHALL create it from the checkout
under test.

#### Scenario: Story Without a Published Release

- **WHEN** a starter story runs for a version that has no published SDK
- **THEN** it creates, builds and runs the starter successfully
