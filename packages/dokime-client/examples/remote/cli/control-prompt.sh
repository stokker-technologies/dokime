#!/bin/sh
# Dokime — SPDX-License-Identifier: Apache-2.0
set -eu

# Connection Parameters
ENDPOINT="https://127.0.0.1:8766"
CA_CERTIFICATE="server.pem"
KEY_FILE="remote.key"
ENV_FILE=""

# Case And Execution Parameters
TEMPLATE="obc.checks.pulse"
CYCLE_LABEL="Remote Confirmation Demonstration"
ANSWER="yes"
WAIT_SECONDS=60

# Evidence Files
OUTPUT_DIRECTORY="prompt-evidence"
OVERWRITE=false

# Select Secrets Once; Each Call Below Shows Its Remote Command.
set -- --endpoint "$ENDPOINT"
if [ -n "$CA_CERTIFICATE" ]; then set -- "$@" --ca "$CA_CERTIFICATE"; fi
if [ -n "$KEY_FILE" ]; then set -- "$@" --key-file "$KEY_FILE"; fi
if [ -n "$ENV_FILE" ]; then set -- "$@" --env-file "$ENV_FILE"; fi

cycle=""
execution=""
destination=""
result=0
case "$WAIT_SECONDS" in ''|*[!0-9]*|0) printf 'WAIT_SECONDS Must Be A Positive Integer\n' >&2; exit 2 ;; esac
mkdir -p "$OUTPUT_DIRECTORY/recovery"
overwrite=""
if [ "$OVERWRITE" = true ]; then overwrite="--overwrite"; fi

# Keep Confirmed UUIDs For Inspection If A Command Fails.
trap 'status=$?; if [ "$status" -ne 0 ]; then printf "Workflow Incomplete; Inspect Cycle: %s Execution: %s Before Resubmitting\n" "$cycle" "$execution" >&2; fi' 0

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
        status=0
        dokime execution wait "$execution" "$@" || status=$?
        case "$status" in 0|1) ;; *) exit "$status" ;; esac
        if [ -n "$destination" ] && [ ! -e "$destination" ]; then
            dokime execution export "$execution" --output "$destination" ${overwrite:+"$overwrite"} "$@"
        fi
    fi
    if [ ! -e "$OUTPUT_DIRECTORY/cycle.zip" ]; then
        dokime cycle export "$cycle" --output "$OUTPUT_DIRECTORY/cycle.zip" ${overwrite:+"$overwrite"} "$@"
    fi
    exit 130
' INT TERM

case "$ANSWER" in yes|no) ;; *) printf 'ANSWER Must Be yes Or no\n' >&2; exit 2 ;; esac

# Preserve Existing execution.zip Before Starting Device Work.
if [ "$OVERWRITE" != true ] && { [ -e "$OUTPUT_DIRECTORY/execution.zip" ] || [ -L "$OUTPUT_DIRECTORY/execution.zip" ]; }; then
    printf 'Evidence File Exists: %s\n' "$OUTPUT_DIRECTORY/execution.zip" >&2
    exit 2
fi

# Preserve Existing cycle.zip Before Starting Device Work.
if [ "$OVERWRITE" != true ] && { [ -e "$OUTPUT_DIRECTORY/cycle.zip" ] || [ -L "$OUTPUT_DIRECTORY/cycle.zip" ]; }; then
    printf 'Evidence File Exists: %s\n' "$OUTPUT_DIRECTORY/cycle.zip" >&2
    exit 2
fi

# Materialize The Immutable Case; Its ID Is Reused By Independent Attempts.
case_id=$(dokime plan materialize --template "$TEMPLATE" --parameters-json '{"require_confirmation":true}' --id-only "$@")
case "$case_id" in ''|*'
'*) printf 'Expected One Case\n' >&2; exit 2 ;; esac
printf 'Case: %s\n' "$case_id"
cycle=$(dokime cycle create --label "$CYCLE_LABEL" --id-only "$@")
printf 'Cycle: %s\n' "$cycle"

# Append One Attempt; Its UUID Scopes The Prompt And Evidence.
execution=$(dokime cycle append "$cycle" --case-id "$case_id" --id-only \
    --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery" "$@")
printf 'Execution: %s\n' "$execution"
destination="$OUTPUT_DIRECTORY/execution.zip"

# Start Without Blocking, Then Wait For This Execution's Next Prompt.
dokime execution start "$execution" --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery" "$@"
prompt=$(dokime execution prompt next "$execution" --wait-seconds "$WAIT_SECONDS" --id-only \
    --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery" "$@")
printf 'Prompt: %s Execution: %s\n' "$prompt" "$execution"
dokime execution prompt show "$execution" "$prompt" --cancel-on-interrupt --owned-cycle "$cycle" \
    --recovery-directory "$OUTPUT_DIRECTORY/recovery" "$@"

# Answer That Prompt Once; Wait For The Durable Result Before Downloading.
dokime execution prompt answer "$execution" "$prompt" --action "$ANSWER" --cancel-on-interrupt --owned-cycle "$cycle" \
    --recovery-directory "$OUTPUT_DIRECTORY/recovery" "$@"
status=0
dokime execution wait "$execution" --wait-seconds "$WAIT_SECONDS" --cancel-on-interrupt --owned-cycle "$cycle" \
    --recovery-directory "$OUTPUT_DIRECTORY/recovery" "$@" || status=$?
case "$status" in 0) ;; 1) result=1 ;; *) exit "$status" ;; esac
dokime execution export "$execution" --output "$OUTPUT_DIRECTORY/execution.zip" \
    ${overwrite:+"$overwrite"} --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery" "$@"

# Seal The Cycle And Download All Members Into cycle.zip.
dokime cycle seal "$cycle" "$@"
dokime cycle export "$cycle" --output "$OUTPUT_DIRECTORY/cycle.zip" ${overwrite:+"$overwrite"} "$@"
exit "$result"
