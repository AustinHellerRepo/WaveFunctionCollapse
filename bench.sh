#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

echo "Running benchmarks..."
cargo bench "$@"

echo ""

REPORT="target/criterion/landscape/report/index.html"

if [[ -f "$REPORT" ]]; then
    echo "Opening benchmark report in browser..."
    xdg-open "$REPORT"
else
    echo "Benchmark report not found at: $REPORT"
fi
