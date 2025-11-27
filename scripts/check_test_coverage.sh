#!/bin/bash
# Health check: Ensure every file with functions has tests

set -e

echo "🔍 Checking test coverage compliance..."

# Files to check (must have corresponding tests)
FILES_WITH_FUNCTIONS=(
    "src/script.rs"
    "src/parser.rs"
    "src/assets.rs"
    "src/renderer/frame_buffer.rs"
    "src/renderer/compositor.rs"
    "src/renderer/timeline.rs"
    "src/renderer/engine.rs"
)

#Check each file has a test module
FAIL=0
for file in "${FILES_WITH_FUNCTIONS[@]}"; do
    if ! grep -q "#\[cfg(test)\]" "$file"; then
        echo "❌ FAIL: $file has no test module"
        FAIL=1
    else
        echo "✅ PASS: $file has test module"
    fi
done

# Run all tests
echo ""
echo "🧪 Running all unit tests..."
cargo test --lib

if [ $FAIL -eq 1 ]; then
    echo ""
    echo "❌ Health check FAILED: Some files lack test coverage"
    exit 1
fi

echo ""
echo "✅ Health check PASSED: All files with functions have tests"
