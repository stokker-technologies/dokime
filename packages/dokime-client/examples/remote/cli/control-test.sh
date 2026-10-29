#!/bin/sh
# Dokime — SPDX-License-Identifier: Apache-2.0
set -eu

# Connection Parameters
BACKEND_IDENTIFIER="lab"

# Case And Execution Parameters
TEMPLATE="obc.checks.pulse"
CYCLE_LABEL="Remote Pulse Demonstration"
SAMPLE_COUNT=8
EXECUTION_COUNT=2
WAIT_SECONDS=60

# Observation Filters
RECORD_TYPES="obc.run.*"
STREAMS="obc.pulse"

# Evidence Files
OUTPUT_DIRECTORY="test-evidence"
OVERWRITE=false

# Select The Imported Profile Once; Each Call Below Shows Its Remote Command.
set -- --backend "$BACKEND_IDENTIFIER"

cycle=""
execution=""
destination=""
case "$WAIT_SECONDS" in ''|*[!0-9]*|0) printf 'WAIT_SECONDS Must Be A Positive Integer\n' >&2; exit 2 ;; esac
mkdir -p "$OUTPUT_DIRECTORY/recovery"
overwrite=""
if [ "$OVERWRITE" = true ]; then overwrite="--overwrite"; fi

# Recover Only This Script-Owned Cycle If The Shell Is Interrupted Between Calls.
trap '
    trap "" INT TERM
    if [ -z "$cycle" ]; then
        printf "Cycle Creation Was Not Confirmed; Inspect The Server Before Resubmitting\n" >&2
        exit 130
    fi
    if [ -n "$execution" ]; then dokime execution stop "$execution" "$@" || :; fi
    dokime cycle seal "$cycle" "$@"
    if [ -n "$execution" ]; then
        dokime execution wait "$execution" "$@"
        if [ -n "$destination" ] && [ ! -e "$destination" ]; then
            dokime execution export "$execution" --output "$destination" ${overwrite:+"$overwrite"} "$@"
        fi
    fi
    if [ ! -e "$OUTPUT_DIRECTORY/cycle.zip" ]; then
        dokime cycle export "$cycle" --output "$OUTPUT_DIRECTORY/cycle.zip" ${overwrite:+"$overwrite"} "$@"
    fi
    exit 130
' INT TERM

# The Story Spells Out Two Attempts; Keep One To Study A Single Run.
case "$EXECUTION_COUNT" in 1|2) ;; *) printf 'EXECUTION_COUNT Must Be 1 Or 2\n' >&2; exit 2 ;; esac

# Preserve Existing execution-1.zip Before Starting Device Work.
if [ "$OVERWRITE" != true ] && { [ -e "$OUTPUT_DIRECTORY/execution-1.zip" ] || [ -L "$OUTPUT_DIRECTORY/execution-1.zip" ]; }; then
    printf 'Evidence File Exists: %s\n' "$OUTPUT_DIRECTORY/execution-1.zip" >&2
    exit 2
fi

# Preserve Existing execution-2.zip Before Starting Device Work.
if [ "$EXECUTION_COUNT" = 2 ] && [ "$OVERWRITE" != true ] && { [ -e "$OUTPUT_DIRECTORY/execution-2.zip" ] || [ -L "$OUTPUT_DIRECTORY/execution-2.zip" ]; }; then
    printf 'Evidence File Exists: %s\n' "$OUTPUT_DIRECTORY/execution-2.zip" >&2
    exit 2
fi

# Preserve Existing cycle.zip Before Starting Device Work.
if [ "$OVERWRITE" != true ] && { [ -e "$OUTPUT_DIRECTORY/cycle.zip" ] || [ -L "$OUTPUT_DIRECTORY/cycle.zip" ]; }; then
    printf 'Evidence File Exists: %s\n' "$OUTPUT_DIRECTORY/cycle.zip" >&2
    exit 2
fi

# Materialize The Immutable Case; Its ID Is Reused By Independent Attempts.
case_id=$(dokime plan materialize --template "$TEMPLATE" --parameters-json "{\"sample_count\":$SAMPLE_COUNT}" --id-only "$@")
case "$case_id" in ''|*'
'*) printf 'Expected One Case\n' >&2; exit 2 ;; esac
printf 'Case: %s\n' "$case_id"
dokime cycle create --label "$CYCLE_LABEL" --json "$@" > cycle.json
cycle=$(sed -n 's/^{"cycle_id":"\([^"]*\)"}$/\1/p' cycle.json)
printf 'Cycle: %s\n' "$cycle"

# Append Both Attempts Without Starting Hardware.
dokime cycle append "$cycle" --case-id "$case_id" --id-only --id-file execution.id --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery" "$@"
IFS= read -r first < execution.id
printf 'Execution: %s\n' "$first"
if [ "$EXECUTION_COUNT" = 2 ]; then
    dokime cycle append "$cycle" --case-id "$case_id" --id-only --id-file execution.id --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery" "$@"
    IFS= read -r second < execution.id
    printf 'Execution: %s\n' "$second"
fi

# Run Attempt 1; The Command Waits For Its Saved Outcome And Listener Drain.
execution="$first"
destination="$OUTPUT_DIRECTORY/execution-1.zip"
dokime execution run "$first" --records --samples --record-type "$RECORD_TYPES" --stream "$STREAMS" --wait-seconds "$WAIT_SECONDS" --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery" "$@"
dokime execution export "$first" --output "$OUTPUT_DIRECTORY/execution-1.zip" ${overwrite:+"$overwrite"} --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery" "$@"

if [ "$EXECUTION_COUNT" = 2 ]; then
    # Run Attempt 2; The Command Waits For Its Saved Outcome And Listener Drain.
    execution="$second"
    destination="$OUTPUT_DIRECTORY/execution-2.zip"
    dokime execution run "$second" --records --samples --record-type "$RECORD_TYPES" --stream "$STREAMS" --wait-seconds "$WAIT_SECONDS" --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery" "$@"
    dokime execution export "$second" --output "$OUTPUT_DIRECTORY/execution-2.zip" ${overwrite:+"$overwrite"} --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery" "$@"
fi

# Seal The Cycle And Download All Members Into cycle.zip.
dokime cycle seal "$cycle" "$@"
dokime cycle export "$cycle" --output "$OUTPUT_DIRECTORY/cycle.zip" ${overwrite:+"$overwrite"} "$@"
