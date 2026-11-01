# step-file-watches Specification

## Purpose
TBD - created by archiving change watchfile-rules. Update Purpose after archive.

## Requirements

### Requirement: Recursive Filtered Step Watches

WatchFile SHALL expose recursive discovery and ordered root-relative FileRule globs through its existing request. Directory requests SHALL preserve relative paths below output, reject a single artifact override, and enforce file, byte and polling bounds. Empty rules with recursion disabled SHALL retain single-file behavior; recursive requests without rules SHALL include all files.

#### Scenario: A Log Tree Grows After Arming
- **WHEN** a Stream watch arms over existing files and a new subfolder later appears
- **THEN** existing files contribute only bytes after the baseline and new files contribute bytes from zero under output/relative-path in both native host and harness

#### Scenario: Archives Are Mirrored Selectively
- **WHEN** a recursive Mirror watch includes **/*.zip
- **THEN** only matching files are captured and changed archives replace their mirrored bytes

#### Scenario: Resource Limits Remain Enforced
- **WHEN** a directory watch exceeds its requested file or byte cap
- **THEN** it fails explicitly and retains captured evidence

### Requirement: Safe Ordinary Artifact Extensions

Artifacts SHALL accept ordinary extensions including .diag and .hpl30 with application/octet-stream fallback for unknown types. Both paths SHALL reject traversal, absolute paths and portable reserved device names.

#### Scenario: Diagnostic Files Are Snapshotted
- **WHEN** a step snapshots .diag and .hpl30 files into the artifact root
- **THEN** both native host and harness retain their original names and bytes

#### Scenario: Unsafe Names Are Refused
- **WHEN** an artifact name contains traversal, an absolute path or a reserved device segment
- **THEN** host and harness reject the request
