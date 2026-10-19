#!/bin/sh
# Dokime — SPDX-License-Identifier: Apache-2.0
set -efu

# Connection Parameters
ENDPOINT="https://127.0.0.1:8766"
CA_CERTIFICATE="server.pem"
KEY_FILE="remote.key"
ENV_FILE=""

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

# Connect With The Explicit Parameters
cli() {
    set -- "$@" --endpoint "$ENDPOINT"
    if [ -n "$CA_CERTIFICATE" ]; then set -- "$@" --ca "$CA_CERTIFICATE"; fi
    if [ -n "$KEY_FILE" ]; then set -- "$@" --key-file "$KEY_FILE"; fi
    if [ -n "$ENV_FILE" ]; then set -- "$@" --env-file "$ENV_FILE"; fi
    dokime "$@"
}

cycle=""
execution=""
destination=""
result=0
case "$WAIT_SECONDS" in ''|*[!0-9]*|0) printf 'WAIT_SECONDS Must Be A Positive Integer\n' >&2; exit 2 ;; esac
mkdir -p "$OUTPUT_DIRECTORY/recovery"

# Retain Confirmed Identities On Failure
trap 'status=$?; if [ "$status" -ne 0 ]; then printf "Workflow Incomplete; Inspect Cycle: %s Execution: %s Before Resubmitting\n" "$cycle" "$execution" >&2; fi' 0

# Preserve Existing Evidence Before Starting Device Work
preflight() {
    if [ "$OVERWRITE" != true ] && { [ -e "$1" ] || [ -L "$1" ]; }; then
        printf 'Evidence File Exists: %s\n' "$1" >&2
        exit 2
    fi
}
export_bundle() {
    if [ "$OVERWRITE" = true ]; then cli "$@" --overwrite; else cli "$@"; fi
}
wait_saved() {
    status=0
    cli "$@" || status=$?
    case "$status" in 0) ;; 1) result=1 ;; *) exit "$status" ;; esac
}

# Recover This Script's Cycle At Any Shell Interruption Boundary
cancel() {
    trap '' INT TERM
    if [ -z "$cycle" ]; then
        printf 'Cycle Creation Was Not Confirmed; Inspect The Server Before Resubmitting\n' >&2
        exit 130
    fi
    if [ -n "$execution" ]; then cli execution stop "$execution" || :; fi
    cli cycle seal "$cycle"
    if [ -n "$execution" ]; then
        wait_saved execution wait "$execution"
        if [ -n "$destination" ] && [ ! -e "$destination" ]; then
            export_bundle execution export "$execution" --output "$destination"
        fi
    fi
    if [ ! -e "$OUTPUT_DIRECTORY/cycle.zip" ]; then
        export_bundle cycle export "$cycle" --output "$OUTPUT_DIRECTORY/cycle.zip"
    fi
    exit 130
}
trap cancel INT TERM

case "$EXECUTION_COUNT" in ''|*[!0-9]*|0) printf 'EXECUTION_COUNT Must Be Positive\n' >&2; exit 2 ;; esac

# Materialize Cases Before Constructing Independent Attempts
cases=$(cli plan materialize --template "$TEMPLATE" --parameters-json "{\"sample_count\":$SAMPLE_COUNT}" --id-only)
[ -n "$cases" ] || { printf 'No Cases Were Materialized\n' >&2; exit 2; }
index=0
attempt=0
while [ "$attempt" -lt "$EXECUTION_COUNT" ]; do
    for case_id in $cases; do
        index=$((index + 1))
        preflight "$OUTPUT_DIRECTORY/execution-$index.zip"
    done
    attempt=$((attempt + 1))
done
preflight "$OUTPUT_DIRECTORY/cycle.zip"

# Append Every Attempt Without Starting
cycle=$(cli cycle create --label "$CYCLE_LABEL" --id-only)
printf 'Cycle: %s\n' "$cycle"
executions=""
attempt=0
while [ "$attempt" -lt "$EXECUTION_COUNT" ]; do
    for case_id in $cases; do
        execution=$(cli cycle append "$cycle" --case-id "$case_id" --id-only \
            --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery")
        printf 'Execution: %s\n' "$execution"
        executions="$executions $execution"
    done
    attempt=$((attempt + 1))
done

# Run Sequentially And Drain Requested Listeners Before Export
index=0
for execution in $executions; do
    index=$((index + 1))
    destination="$OUTPUT_DIRECTORY/execution-$index.zip"
    set -- execution run "$execution" --records --samples --wait-seconds "$WAIT_SECONDS" \
        --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery"
    for record_type in $RECORD_TYPES; do set -- "$@" --record-type "$record_type"; done
    for stream in $STREAMS; do set -- "$@" --stream "$stream"; done
    wait_saved "$@"
    export_bundle execution export "$execution" --output "$destination" \
        --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery"
done

# Seal The Container And Save Its Complete Bundle
cli cycle seal "$cycle"
export_bundle cycle export "$cycle" --output "$OUTPUT_DIRECTORY/cycle.zip"
exit "$result"
