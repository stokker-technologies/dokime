# File Capture Specification

## Purpose
Define how a step brings files into its execution's evidence: copying a source once, or capturing it between ordinary start and stop operations as a live Stream or a whole-file Mirror, with gaps and failures recorded as evidence.

## Requirements

### Requirement: One-Shot File Copy

Dokime SHALL offer steps a `CopyFiles` operation that copies its source into execution artifacts once, settles with what it copied, and needs no later call.

#### Scenario: Cancelled Copy Seals
- **WHEN** a `CopyFiles` call is cancelled while its files are sealing
- **THEN** it answers once they have sealed, within its attempt deadline rather than the shorter run-stop reply budget

#### Scenario: Newest File Is Copied at Start
- **WHEN** a step calls `CopyFiles` with a `Newest` source over a folder holding several matching files
- **THEN** only the newest match by modification time, breaking ties by path, becomes an artifact under the output prefix and the response lists it

#### Scenario: Tree Is Copied
- **WHEN** a step calls `CopyFiles` with a `Tree` source
- **THEN** every selected file becomes an artifact under the output prefix with its relative path preserved

### Requirement: Capture Starts and Stops With Ordinary Operations

Dokime SHALL offer `StartCapture` and `StopCapture` as ordinary request and response operations. `StartCapture` SHALL return a capture identifier once the baseline is taken. `StopCapture` SHALL take a final pass, seal and return the captured artifacts and gaps without passing through operation cancellation.

#### Scenario: Producer Starts After the Baseline
- **WHEN** `StartCapture` has returned and a producer then writes to the source
- **THEN** those bytes are captured with no startup wait in the caller

#### Scenario: Stop Returns the Outcome After a Slow Seal
- **WHEN** a capture over many files is stopped and sealing outlasts the operation's `cancellation_timeout`
- **THEN** `StopCapture` returns the complete outcome rather than a cancellation

#### Scenario: Seal Failure Reaches the Caller
- **WHEN** sealing fails during `StopCapture`
- **THEN** the call returns that failure and what was captured is retained

#### Scenario: Repeated Stop
- **WHEN** `StopCapture` is called again for a capture that has already stopped
- **THEN** it returns the same outcome and captures nothing more

#### Scenario: Abandoned Stop Reply
- **WHEN** a `StopCapture` call is cancelled while sealing is in progress
- **THEN** sealing completes, every artifact is retained, and a later `StopCapture` returns the outcome

#### Scenario: Unknown or Foreign Capture
- **WHEN** `StopCapture` names an unknown capture or one owned by another execution
- **THEN** the call is refused and the capture is unaffected

### Requirement: Execution-Owned Captures

A capture SHALL belong to the calling step's execution and SHALL continue across that execution's later steps until it is stopped or the execution seals it. A capture left running SHALL seal at execution end, retaining its evidence, without a warning. A call without an execution in its context SHALL be refused.

#### Scenario: Capture Spans Steps
- **WHEN** one step starts a capture and a later step of the same execution stops it
- **THEN** bytes written between those steps are captured and the later step receives the outcome

#### Scenario: Capture Is Never Stopped
- **WHEN** an execution ends with a capture still running
- **THEN** a final pass runs, its artifacts seal before the execution's own durable seal, and no warning is recorded

### Requirement: File Source Shapes

Every copy and capture request SHALL name exactly one `FileSource`: `File` for one file, `Tree` for a directory with ordered root-relative rules, or `Newest` for the newest match of such a selection at the start. The request SHALL NOT carry a recursion flag. `File` output SHALL be the full artifact name; `Tree` and `Newest` output SHALL be a prefix that preserves relative paths.

#### Scenario: Missing File or Root
- **WHEN** a `File` source or the root of a `Tree` or `Newest` source does not exist at the start
- **THEN** the request fails immediately with an explicit error

#### Scenario: Empty Tree Arms
- **WHEN** a `Tree` source names an accessible directory with no matching file
- **THEN** the capture starts at once and a later matching file is captured from byte zero

#### Scenario: Newest Finds Nothing
- **WHEN** a `Newest` source has no matching file at the start
- **THEN** the request fails explicitly rather than capturing nothing

#### Scenario: Newest Never Switches
- **WHEN** a capture follows a `Newest` file and a newer file later appears
- **THEN** the capture keeps following its selected file, including through rotation

#### Scenario: Rules Decide Depth
- **WHEN** a `Tree` source has rule `*.log` and another has rule `**/*.log`
- **THEN** the first selects only top-level files and the second selects files at any depth, with no separate flag

#### Scenario: Empty Rules
- **WHEN** a `Tree` source has no rules
- **THEN** every file at any depth is selected

### Requirement: Stream Capture

`Stream` mode SHALL capture bytes written after the baseline: files present at the start from their baseline length and later files from byte zero. A truncated or replaced file SHALL seal its segment and continue in a separately named segment. Line matchers and markers SHALL be accepted only by `Stream` and SHALL publish rows and records as lines are captured. Captured files SHALL appear live with their growing size and bounded readable content.

#### Scenario: Live Log Is Followed
- **WHEN** a producer appends to a streamed log while a viewer follows its artifact
- **THEN** the viewer sees the appended bytes before the capture stops, and matching lines have published their rows and records

#### Scenario: Untouched Baseline
- **WHEN** a `Stream` capture stops without changes to its existing files
- **THEN** no artifact is retained and the outcome lists none

#### Scenario: Log Rotates
- **WHEN** a streamed file is renamed and a new file takes its path
- **THEN** both segments are retained under distinct names with a gap recorded between them

### Requirement: Mirror Capture

`Mirror` mode SHALL keep each output equal to its source, checking for a size or modification-time change once per interval and ignoring native change notifications. The interval SHALL default from configuration and MAY be overridden per request within configured bounds. A final pass SHALL run when the capture stops or the execution seals it. A failed refetch SHALL keep the last whole copy.

#### Scenario: Zip Changes During the Execution
- **WHEN** a zip file under a 10 s `Mirror` capture is rewritten several times within one interval
- **THEN** at most one refetch runs in that interval, and the sealed artifact holds the bytes present at the final pass

#### Scenario: Change After the Last Interval
- **WHEN** the source changes and the capture stops before the next interval elapses
- **THEN** the final pass captures that change

#### Scenario: Refetch Fails
- **WHEN** a changed source cannot be read
- **THEN** the output keeps its last whole copy and never holds a partial or emptied file

#### Scenario: Line Rules Belong to Stream
- **WHEN** an author builds a `Mirror` request
- **THEN** the request has no field for line matchers or markers

### Requirement: Capture Gaps Are Evidence

Every source gap SHALL be published as a warning record of type `dokime.files.capture_gap` in the capture's execution scope, carrying the path, output offset and reason, whether or not any caller reads the outcome.

#### Scenario: Truncation With No Stop
- **WHEN** a captured file is truncated and the execution seals the capture without a `StopCapture` call
- **THEN** the execution's evidence holds a gap record naming that file, its offset and the reason

#### Scenario: Stop Reports Gaps
- **WHEN** a capture with gaps is stopped
- **THEN** `StopCapture` also lists those gaps in its response

### Requirement: Capture Failures Are Evidence

A capture that ends early SHALL publish a warning record of type `dokime.files.capture_failed` in the capture's execution scope carrying the reason, whether or not any caller stops it. `StopCapture` SHALL also return the failure.

#### Scenario: Capture Fails With No Stop
- **WHEN** a running capture exceeds a limit and nothing stops it
- **THEN** the execution's evidence holds a failure record with the reason, and what the capture held seals

### Requirement: FTP Behind a Feature

The FTP implementation and its dependencies SHALL compile only with the `ftp` feature, which `native` SHALL NOT include and shipped server, desktop and CLI builds SHALL enable. The wire shape SHALL NOT depend on the feature. A build without the feature SHALL refuse an FTP location with the `UNSUPPORTED` code. With the feature, FTP sources SHALL keep their landed trust, secret-store, password-refusal and resume behavior.

#### Scenario: Plugin Sidecar Builds Without FTP
- **WHEN** a plugin depends only on `dokime/plugin-sidecar`
- **THEN** its dependency graph excludes `suppaftp`, and its operation contracts equal those of a host built with `ftp`

#### Scenario: FTP Refused Without the Feature
- **WHEN** a request names an FTP location on a build without `ftp`
- **THEN** it is refused with `UNSUPPORTED` before any connection

#### Scenario: Shipped Host Reads FTP
- **WHEN** a step requests an FTP source from a shipped host
- **THEN** the source opens under the landed trust and secret-store rules

### Requirement: Capture Limits Come From Configuration

Requests SHALL NOT carry file limits, byte limits or a `Stream` poll interval. Those values and the capture capacity SHALL come from typed configuration with one default each. Exceeding a limit SHALL fail explicitly and retain captured evidence.

#### Scenario: Byte Limit Reached
- **WHEN** a capture exceeds the configured byte limit
- **THEN** it fails explicitly and what it captured seals

#### Scenario: Configuration Raises a Limit
- **WHEN** configuration raises the byte limit above a source's size
- **THEN** the same request captures the whole source with no request change
