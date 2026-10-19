#!/bin/sh
# Dokime — SPDX-License-Identifier: Apache-2.0
# Pass Connection Flags Such As --endpoint, --ca, --key-file Or --env-file.
set -eu

dokime=${DOKIME_EXECUTABLE:-dokime}
directory=${DOKIME_OUTPUT_DIRECTORY:-prompt-evidence}
cycle=
execution=
prompt=
result=0

mkdir -p "$directory"
for name in execution.zip cycle.zip interrupted-execution.zip interrupted-cycle.zip; do
    destination=$directory/$name
    if [ -e "$destination" ] || [ -L "$destination" ]; then
        printf 'Evidence File Exists: %s; Choose An Empty Output Directory\n' "$destination" >&2
        exit 2
    fi
done

recover() {
    status=$?
    if [ "$status" -ne 0 ]; then
        printf 'Recovery Cycle: %s Execution: %s Prompt: %s\n' "$cycle" "$execution" "$prompt" >&2
        printf 'A Failed Connection Does Not Confirm Cancellation Or An Answer. Inspect The Original Identities Before Resubmitting\n' >&2
    fi
}
trap recover 0

# Exit 1 Means A Saved Non-Passing Outcome; Other Errors Cannot Prove Sealing.
wait_saved() {
    wait_status=0
    "$dokime" execution wait "$execution" --wait-seconds 120 "$@" || wait_status=$?
    case "$wait_status" in
        0) return 0 ;;
        1) result=1; return 0 ;;
        *) return "$wait_status" ;;
    esac
}

cancel() {
    trap '' INT TERM
    printf 'Cancellation Requested; Waiting For Saved Evidence\n' >&2
    if [ -z "$cycle" ]; then
        printf 'Cycle Creation Was Not Confirmed. Inspect The Host Before Creating Another\n' >&2
        exit 130
    fi
    if [ -n "$execution" ]; then
        "$dokime" execution stop "$execution" "$@" || printf 'Stop Was Not Confirmed; Checking For A Durable Outcome\n' >&2
        wait_saved "$@"
        "$dokime" execution export "$execution" --output "$directory/interrupted-execution.zip" "$@"
    fi
    "$dokime" cycle seal "$cycle" "$@"
    "$dokime" cycle export "$cycle" --output "$directory/interrupted-cycle.zip" "$@"
    exit 130
}
trap 'cancel "$@"' INT TERM

cycle=$("$dokime" cycle create --label 'Remote Confirmation Demonstration' --id-only "$@")
printf 'Cycle: %s\n' "$cycle"
execution=$("$dokime" cycle append "$cycle" --template obc.checks.pulse --parameters-json '{"require_confirmation":true}' --id-only "$@")
request_key=remote-$execution
printf 'Execution: %s Request Key: %s Mode: automatic\n' "$execution" "$request_key"
"$dokime" execution run "$execution" --request-key "$request_key" "$@"

prompt=$("$dokime" execution prompt next "$execution" --wait-seconds 60 --id-only "$@")
printf 'Prompt: %s Execution: %s\n' "$prompt" "$execution"
"$dokime" execution prompt show "$execution" "$prompt" "$@"
"$dokime" execution prompt answer "$execution" "$prompt" --action yes "$@"
wait_saved "$@"
"$dokime" execution export "$execution" --output "$directory/execution.zip" "$@"
"$dokime" cycle seal "$cycle" "$@"
"$dokime" cycle export "$cycle" --output "$directory/cycle.zip" "$@"
exit "$result"
