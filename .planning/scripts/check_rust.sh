#!/usr/bin/env bash
set -uo pipefail

# Fast Rust pattern checker - lightweight alternative to rust-analyzer
# Uses regex patterns to catch common issues without full compilation

RULES_FILE="${RULES_FILE:-/opt/claude-code-marketplace/plugins/coding-standards-enforcer/rules/rust/rules.yaml}"
EXIT_CODE=0
WARNINGS=0
ERRORS=0
CRITICAL=0

# Color output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m'

# Check if file provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <rust-file.rs>"
    exit 1
fi

RUST_FILE="$1"

if [ ! -f "$RUST_FILE" ]; then
    echo "Error: File not found: $RUST_FILE"
    exit 1
fi

# Detect if this is a test file (strict naming convention)
# Test files MUST either:
#   1. Be in tests/ directory (standard Rust convention), OR
#   2. End with _test.rs (makes tests visually obvious)
IS_TEST_FILE=0
if [[ "$RUST_FILE" =~ /tests?/ ]] || [[ "$RUST_FILE" =~ _test\.rs$ ]]; then
    IS_TEST_FILE=1
fi

# Critical patterns (will fail the check)
# Note: unwrap/expect/panic are allowed in test files (standard practice)
declare -A CRITICAL_PATTERNS=(
    ["unsafe"]='\bunsafe\s+(fn|impl|trait|\{|struct|enum)'
    ["todo"]='(TODO|FIXME|HACK|XXX|TEMP|WIP|PLACEHOLDER)'
    ["mock"]='(mock|Mock|fake|Fake|stub|Stub|dummy|Dummy)\s*(struct|impl|fn|mod)'
    ["unimplemented"]='\bunimplemented!\s*\('
    ["allow_dead_code"]='#\[allow\(dead_code\)\]'
    ["allow_unused"]='#\[allow\(unused'
    ["dbg_macro"]='\bdbg!\s*\('
)

# Additional patterns only checked in production code (not tests)
declare -A PRODUCTION_ONLY_PATTERNS=(
    ["unwrap"]='\.unwrap\s*\('
    ["expect"]='\.expect\s*\('
    ["panic"]='\bpanic!\s*\('
)

# Error patterns (should be fixed)
declare -A ERROR_PATTERNS=(
    ["clone_on_copy"]='\.clone\(\)'
    ["unwrap_or_default"]='\.unwrap_or_default\(\)'
    ["as_str_to_string"]='\.as_str\(\)\.to_string\(\)'
)

# Warning patterns (should be reviewed)
declare -A WARNING_PATTERNS=(
    ["missing_doc_comment"]=''
    ["long_function"]=''
)


check_pattern() {
    local pattern="$1"
    local severity="$2"
    local description="$3"
    local file="$4"

    if grep -Pq "$pattern" "$file" 2>/dev/null; then
        local line_numbers=$(grep -Pn "$pattern" "$file" 2>/dev/null | cut -d: -f1 | tr '\n' ',' | sed 's/,$//')

        case "$severity" in
            critical)
                echo -e "${RED}🚨 CRITICAL${NC}: $description"
                echo "   File: $file"
                echo "   Lines: $line_numbers"
                ((CRITICAL++))
                EXIT_CODE=1
                ;;
            error)
                echo -e "${RED}❌ ERROR${NC}: $description"
                echo "   File: $file"
                echo "   Lines: $line_numbers"
                ((ERRORS++))
                EXIT_CODE=1
                ;;
            warning)
                echo -e "${YELLOW}⚠️  WARNING${NC}: $description"
                echo "   File: $file"
                echo "   Lines: $line_numbers"
                ((WARNINGS++))
                EXIT_CODE=1
                ;;
        esac
    fi
}

echo "🔍 Fast-checking Rust file: $RUST_FILE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check file length (200 line limit)
LINE_COUNT=$(wc -l < "$RUST_FILE")
if [ $LINE_COUNT -gt 200 ]; then
    echo -e "${RED}🚨 CRITICAL${NC}: File exceeds 200 lines"
    echo "   File: $RUST_FILE"
    echo "   Lines: $LINE_COUNT (limit: 200)"
    echo "   Split this file into smaller modules"
    ((CRITICAL++))
    EXIT_CODE=1
fi

# Check critical patterns
for desc in "${!CRITICAL_PATTERNS[@]}"; do
    pattern="${CRITICAL_PATTERNS[$desc]}"
    check_pattern "$pattern" "critical" "No $desc allowed" "$RUST_FILE"
done

# Check production-only patterns (skip for test files)
if [ $IS_TEST_FILE -eq 0 ]; then
    for desc in "${!PRODUCTION_ONLY_PATTERNS[@]}"; do
        pattern="${PRODUCTION_ONLY_PATTERNS[$desc]}"
        check_pattern "$pattern" "critical" "No $desc allowed in production code" "$RUST_FILE"
    done
else
    echo "ℹ️  Test file detected - allowing unwrap()/expect()/panic!() as standard practice"
fi

# Check println! outside of main.rs and tests
if [[ ! "$RUST_FILE" =~ main\.rs$ ]] && [[ ! "$RUST_FILE" =~ /tests?/ ]] && [[ ! "$RUST_FILE" =~ _test\.rs$ ]]; then
    if grep -Pq '\bprintln!\s*\(' "$RUST_FILE" 2>/dev/null; then
        line_numbers=$(grep -Pn '\bprintln!\s*\(' "$RUST_FILE" 2>/dev/null | cut -d: -f1 | tr '\n' ',' | sed 's/,$//')
        echo -e "${RED}❌ ERROR${NC}: println! should only be in main.rs or tests"
        echo "   File: $RUST_FILE"
        echo "   Lines: $line_numbers"
        echo "   Use proper logging (tracing, log, etc.) instead"
        ((ERRORS++))
        EXIT_CODE=1
    fi
fi

# Check for single-letter variable names (except i, j, k in loops, x/y/z in math)
if grep -Pn '^\s*let\s+[a-hln-wA-Z]\s*[=:]' "$RUST_FILE" 2>/dev/null | grep -v '^\s*let\s*[ijk]\s*[=:]' | grep -v '^\s*let\s*[xyz]\s*[=:]' > /dev/null; then
    line_numbers=$(grep -Pn '^\s*let\s+[a-hln-wA-Z]\s*[=:]' "$RUST_FILE" 2>/dev/null | grep -v '\s*[ijk]\s*[=:]' | grep -v '\s*[xyz]\s*[=:]' | cut -d: -f1 | tr '\n' ',' | sed 's/,$//')
    echo -e "${YELLOW}⚠️  WARNING${NC}: Single-letter variable names detected"
    echo "   File: $RUST_FILE"
    echo "   Lines: $line_numbers"
    echo "   Use descriptive names (except i,j,k for loops or x,y,z for math)"
    ((WARNINGS++))
    EXIT_CODE=1
fi

# Check function length (50 line limit)
while read -r line; do
    if [[ -n "$line" ]]; then
        echo -e "${RED}🚨 CRITICAL${NC}: $line"
        echo "   File: $RUST_FILE"
        echo "   Limit: 50 lines per function"
        echo "   Split into smaller functions"
        ((CRITICAL++))
        EXIT_CODE=1
    fi
done < <(awk '
/^[[:space:]]*fn [a-zA-Z_]/ {
    fn_start = NR
    fn_name = $0
    brace_count = 0
    in_function = 1
}
in_function && /{/ { brace_count++ }
in_function && /}/ {
    brace_count--
    if (brace_count == 0) {
        fn_length = NR - fn_start + 1
        if (fn_length > 50) {
            printf "Function too long: line %d, length %d lines\n", fn_start, fn_length
        }
        in_function = 0
    }
}
' "$RUST_FILE")

# Check for missing doc comments on pub items
while read -r line_num; do
    if [[ -n "$line_num" ]]; then
        echo -e "${YELLOW}⚠️  WARNING${NC}: missing_doc_comment detected"
        echo "   File: $RUST_FILE"
        echo "   Lines: $line_num"
        ((WARNINGS++))
        EXIT_CODE=1
    fi
done < <(awk '
BEGIN { has_doc = 0; in_attr = 0 }
/^[[:space:]]*\/\/\// { has_doc = 1; next }
/^[[:space:]]*#\[/ {
    if (!/\][[:space:]]*$/) { in_attr = 1 }
    next
}
in_attr {
    if (/\][[:space:]]*$/) { in_attr = 0 }
    next
}
/^[[:space:]]*$/ { has_doc = 0; next }
/^[[:space:]]*pub[[:space:]]+(async[[:space:]]+|const[[:space:]]+)*(fn|struct|enum|trait|type)[[:space:]]/ {
    if (!has_doc) {
        printf "%d\n", NR
    }
    has_doc = 0
    next
}
{ has_doc = 0 }
' "$RUST_FILE")

# Check error patterns
for desc in "${!ERROR_PATTERNS[@]}"; do
    pattern="${ERROR_PATTERNS[$desc]}"
    check_pattern "$pattern" "error" "$desc should be avoided" "$RUST_FILE"
done

# Check warning patterns (only if no critical issues)
if [ $CRITICAL -eq 0 ]; then
    for desc in "${!WARNING_PATTERNS[@]}"; do
        pattern="${WARNING_PATTERNS[$desc]}"
        [ -n "$pattern" ] && check_pattern "$pattern" "warning" "$desc detected" "$RUST_FILE"
    done
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Summary:"
echo "   🚨 Critical: $CRITICAL"
echo "   ❌ Errors: $ERRORS"
echo "   ⚠️  Warnings: $WARNINGS"

if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ Check passed!${NC}"
else
    echo -e "${RED}❌ Check FAILED - fix critical issues${NC}"
fi

exit $EXIT_CODE
