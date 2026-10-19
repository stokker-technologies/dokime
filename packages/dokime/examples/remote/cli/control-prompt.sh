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
        if [ ! -e "$OUTPUT_DIRECTORY/execution.zip" ]; then
            export_bundle execution export "$execution" --output "$OUTPUT_DIRECTORY/execution.zip"
        fi
    fi
    if [ ! -e "$OUTPUT_DIRECTORY/cycle.zip" ]; then
        export_bundle cycle export "$cycle" --output "$OUTPUT_DIRECTORY/cycle.zip"
    fi
    exit 130
}
trap cancel INT TERM

case "$ANSWER" in yes|no) ;; *) printf 'ANSWER Must Be yes Or no\n' >&2; exit 2 ;; esac
preflight "$OUTPUT_DIRECTORY/execution.zip"
preflight "$OUTPUT_DIRECTORY/cycle.zip"

# Materialize The Confirmation Case And Append Its Attempt
cases=$(cli plan materialize --template "$TEMPLATE" --parameters-json '{"require_confirmation":true}' --id-only)
set -- $cases
[ "$#" -eq 1 ] || { printf 'Expected One Confirmation Case\n' >&2; exit 2; }
case_id=$1
cycle=$(cli cycle create --label "$CYCLE_LABEL" --id-only)
printf 'Cycle: %s\n' "$cycle"
execution=$(cli cycle append "$cycle" --case-id "$case_id" --id-only \
            --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery")
printf 'Execution: %s\n' "$execution"

# Start Without Blocking So The Script Can Answer Its Prompt
cli execution start "$execution" \
    --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery"
prompt=$(cli execution prompt next "$execution" --wait-seconds "$WAIT_SECONDS" --id-only \
    --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery")
printf 'Prompt: %s Execution: %s\n' "$prompt" "$execution"
cli execution prompt show "$execution" "$prompt" \
    --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery"
cli execution prompt answer "$execution" "$prompt" --action "$ANSWER" \
    --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery"
wait_saved execution wait "$execution" --wait-seconds "$WAIT_SECONDS" \
    --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery"
export_bundle execution export "$execution" --output "$OUTPUT_DIRECTORY/execution.zip" \
        --cancel-on-interrupt --owned-cycle "$cycle" --recovery-directory "$OUTPUT_DIRECTORY/recovery"

# Seal The Container And Save Its Complete Bundle
cli cycle seal "$cycle"
export_bundle cycle export "$cycle" --output "$OUTPUT_DIRECTORY/cycle.zip"
exit "$result"
