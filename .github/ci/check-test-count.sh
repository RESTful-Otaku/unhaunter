#!/usr/bin/env bash
# Fails if the workspace test count drops below .github/ci/test-baseline.txt,
# so a refactor cannot silently delete tests.
#
# Raise the baseline only when removing tests is intentional, and say why in the
# PR description.
set -euo pipefail
cd "$(dirname "$0")/../.."

BASELINE=$(cat .github/ci/test-baseline.txt)

set +e
OUT=$(cargo test --workspace --all-features 2>&1)
STATUS=$?
set -e
echo "$OUT"

if [ "$STATUS" -ne 0 ]; then
  echo "::error::cargo test failed"
  exit "$STATUS"
fi

PASSED=$(echo "$OUT" | awk '/^test result:/ {p+=$4} END {print p+0}')

if [ "$PASSED" -lt "$BASELINE" ]; then
  echo "::error::tests regressed: $PASSED passed, baseline is $BASELINE."
  echo "::error::If intentional, set .github/ci/test-baseline.txt to $PASSED and explain why."
  exit 1
fi

echo "tests: $PASSED passed (baseline $BASELINE)"
