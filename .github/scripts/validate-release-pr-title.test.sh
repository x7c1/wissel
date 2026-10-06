#!/usr/bin/env bash
set -uo pipefail

# Unit tests for validate-release-pr-title.sh.
#
# Usage:
#   bash validate-release-pr-title.test.sh
#
# One case per row of the allowed-transitions table in
# docs/guides/30-release.md, plus the no-tag baseline. Each case asserts on
# the validator's exit code only: 0 for an accepted title, 1 for a rejected
# one. The `::error::` text is for humans reading the workflow log and is
# free to change.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VALIDATOR="${SCRIPT_DIR}/validate-release-pr-title.sh"

failures=0

pass() {
    printf 'ok   - %s\n' "$1"
}

fail() {
    printf 'FAIL - %s\n' "$1"
    printf '%s\n' "$2"
    failures=$((failures + 1))
}

assert_exit() {
    local name="$1" expected="$2" title="$3" last_tag="$4"
    local output actual

    output=$(bash "$VALIDATOR" "$title" "$last_tag" 2>&1)
    actual=$?
    if [ "$expected" -eq "$actual" ]; then
        pass "$name"
    else
        fail "$name" "  expected exit ${expected}, got ${actual} for $(printf '%q' "$title") against $(printf '%q' "$last_tag")
  output: ${output}"
    fi
}

assert_accepted() {
    assert_exit "$1" 0 "$2" "$3"
}

assert_rejected() {
    assert_exit "$1" 1 "$2" "$3"
}

# --- allowed transitions table ----------------------------------------------

assert_accepted "patch bump is accepted" 'Release v0.3.11' 'v0.3.10'
assert_accepted "minor bump is accepted" 'Release v0.4.0' 'v0.3.10'
assert_accepted "major bump is accepted" 'Release v1.0.0' 'v0.3.10'
assert_rejected "downgrade is rejected" 'Release v0.3.9' 'v0.3.10'
assert_rejected "minor bump with non-zero patch is rejected" 'Release v0.4.1' 'v0.3.10'
assert_rejected "minor skip is rejected" 'Release v0.5.0' 'v0.3.10'
assert_rejected "major skip is rejected" 'Release v2.0.0' 'v0.3.10'
assert_rejected "malformed title is rejected" 'Release 0.3.11' 'v0.3.10'

# --- no-tag baseline --------------------------------------------------------

# With no tag yet the validator measures against v0.0.0, so the first
# release may be a patch, minor or major step from there.
assert_accepted "first patch release from the v0.0.0 baseline is accepted" 'Release v0.0.1' ''
assert_accepted "first minor release from the v0.0.0 baseline is accepted" 'Release v0.1.0' ''
assert_accepted "first major release from the v0.0.0 baseline is accepted" 'Release v1.0.0' ''
assert_rejected "a skip from the v0.0.0 baseline is rejected" 'Release v0.2.0' ''

if [ "$failures" -ne 0 ]; then
    printf '\n%d test(s) failed.\n' "$failures"
    exit 1
fi

printf '\nAll validate-release-pr-title tests passed.\n'
