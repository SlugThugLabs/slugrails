#!/usr/bin/env bash
# Automated repository invariants verification for CodeGuards / SlugRails.
#
# Validates the four foundational architectural and quality invariants:
# 1. Every file in `src/` has <= 200 LOC (hard ceiling).
# 2. Zero occurrences of `unsafe` code in `src/` (enforced by `#![forbid(unsafe_code)]`).
# 3. Zero test pollution in `src/` (no `#[cfg(test)]` or `#[test]` in `src/`).
# 4. All modules declared in `src/guards/checks/mod.rs` exist on disk.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
SRC_DIR="${REPO_ROOT}/src"

echo -e "\033[36m=================================================================\033[0m"
echo -e "\033[36m  SlugRails Repository Invariant Verification (POSIX Shell)\033[0m"
echo -e "\033[90m  Repository Root: ${REPO_ROOT}\033[0m"
echo -e "\033[36m=================================================================\033[0m"
echo ""

python3 - <<'EOF'
import os, sys, re

repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__) if "__file__" in dir() else ".", ".."))
if not os.path.exists(os.path.join(repo_root, "src")):
    repo_root = os.getcwd()
src_dir = os.path.join(repo_root, "src")
violations = 0

def strip_comments_and_strings(source: str) -> str:
    # State machine to strip line comments, block comments, and string literals
    chars = list(source)
    n = len(chars)
    sb = []
    i = 0
    state = "Normal"
    raw_hashes = 0
    while i < n:
        c = chars[i]
        nxt = chars[i+1] if i + 1 < n else ""
        if state == "Normal":
            if c == "/" and nxt == "/":
                state = "LineComment"
                i += 2
                continue
            elif c == "/" and nxt == "*":
                state = "BlockComment"
                i += 2
                continue
            elif c == "r" and (nxt == "#" or nxt == '"'):
                hashes = 0
                idx = i + 1
                while idx < n and chars[idx] == "#":
                    hashes += 1
                    idx += 1
                if idx < n and chars[idx] == '"':
                    state = "RawString"
                    raw_hashes = hashes
                    i = idx + 1
                    continue
                else:
                    sb.append(c)
            elif c == '"':
                state = "String"
            elif c == "'":
                state = "Char"
            else:
                sb.append(c)
        elif state == "LineComment":
            if c == "\n":
                state = "Normal"
                sb.append(c)
        elif state == "BlockComment":
            if c == "*" and nxt == "/":
                state = "Normal"
                i += 2
                continue
            elif c == "\n":
                sb.append(c)
        elif state == "String":
            if c == "\\":
                i += 2
                continue
            elif c == '"':
                state = "Normal"
        elif state == "Char":
            if c == "\\":
                i += 2
                continue
            elif c == "'":
                state = "Normal"
        elif state == "RawString":
            if c == '"':
                match = True
                for h in range(raw_hashes):
                    if i + 1 + h >= n or chars[i + 1 + h] != "#":
                        match = False
                        break
                if match:
                    state = "Normal"
                    i += 1 + raw_hashes
                    continue
        i += 1
    return "".join(sb)

# Invariant 1: File Line Budget (<= 200 LOC per file in src/)
sys.stdout.write("[1/4] Checking File Line Budget (<= 200 LOC)... ")
loc_violations = []
max_lines = 0
max_file = ""
all_src_files = []

for root, _, files in os.walk(src_dir):
    for f in files:
        if f.endswith(".rs"):
            p = os.path.join(root, f)
            all_src_files.append(p)
            with open(p, "r", encoding="utf-8", errors="replace") as fp:
                count = len(fp.readlines())
            rel = os.path.relpath(p, repo_root)
            if count > max_lines:
                max_lines = count
                max_file = rel
            if count > 200:
                loc_violations.append((rel, count))

if not loc_violations:
    print(f"\033[32mPASS\033[0m\n      Scanned {len(all_src_files)} files. Max lines: {max_lines} ({max_file})")
else:
    print("\033[31mFAIL\033[0m")
    for rel, count in loc_violations:
        print(f"      [VIOLATION] {rel} has {count} lines (> 200 LOC ceiling)")
    violations += len(loc_violations)

# Invariant 2: Zero occurrences of unsafe code in src/
sys.stdout.write("[2/4] Checking Zero Unsafe Code & #![forbid(unsafe_code)]... ")
unsafe_violations = []
lib_rs = os.path.join(src_dir, "lib.rs")
main_rs = os.path.join(src_dir, "main.rs")

if os.path.exists(lib_rs):
    with open(lib_rs, "r") as fp:
        if "#![forbid(unsafe_code)]" not in fp.read():
            unsafe_violations.append("src/lib.rs is missing crate-level '#![forbid(unsafe_code)]'")
if os.path.exists(main_rs):
    with open(main_rs, "r") as fp:
        if "#![forbid(unsafe_code)]" not in fp.read():
            unsafe_violations.append("src/main.rs is missing crate-level '#![forbid(unsafe_code)]'")

for f in all_src_files:
    with open(f, "r", encoding="utf-8", errors="replace") as fp:
        raw = fp.read()
    stripped = strip_comments_and_strings(raw)
    if re.search(r"\bunsafe\b", stripped):
        rel = os.path.relpath(f, repo_root)
        unsafe_violations.append(f"Unsafe code token detected in {rel}")

if not unsafe_violations:
    print("\033[32mPASS\033[0m\n      #![forbid(unsafe_code)] present; zero unsafe blocks detected.")
else:
    print("\033[31mFAIL\033[0m")
    for uv in unsafe_violations:
        print(f"      [VIOLATION] {uv}")
    violations += len(unsafe_violations)

# Invariant 3: Zero test pollution in src/
sys.stdout.write("[3/4] Checking Zero Test Pollution in src/... ")
test_violations = []
for f in all_src_files:
    with open(f, "r", encoding="utf-8", errors="replace") as fp:
        raw = fp.read()
    stripped = strip_comments_and_strings(raw)
    if re.search(r"#\[\s*(?:cfg\s*\(\s*test\s*\)|test)\s*\]", stripped):
        rel = os.path.relpath(f, repo_root)
        test_violations.append(f"Test attribute detected in production source: {rel}")

if not test_violations:
    print("\033[32mPASS\033[0m\n      Zero #[cfg(test)] or #[test] attributes in src/ (isolated to tests/).")
else:
    print("\033[31mFAIL\033[0m")
    for tv in test_violations:
        print(f"      [VIOLATION] {tv}")
    violations += len(test_violations)

# Invariant 4: Module Disk Alignment
sys.stdout.write("[4/4] Checking Module Disk Alignment for checks/mod.rs... ")
checks_mod = os.path.join(src_dir, "guards", "checks", "mod.rs")
align_fails = []
if os.path.exists(checks_mod):
    with open(checks_mod, "r") as fp:
        content = fp.read()
    for m in re.finditer(r"(?m)^\s*(?:pub\s+)?mod\s+([a-zA-Z0-9_]+)\s*;", content):
        mod_name = m.group(1)
        target1 = os.path.join(src_dir, "guards", "checks", f"{mod_name}.rs")
        target2 = os.path.join(src_dir, "guards", "checks", mod_name, "mod.rs")
        if not os.path.exists(target1) and not os.path.exists(target2):
            align_fails.append(f"Module '{mod_name}' declared in mod.rs does not exist on disk")

if not align_fails:
    print("\033[32mPASS\033[0m\n      All declared check modules exist on disk.")
else:
    print("\033[31mFAIL\033[0m")
    for af in align_fails:
        print(f"      [VIOLATION] {af}")
    violations += len(align_fails)

print("")
print("=================================================================")
if violations == 0:
    print("\033[32m[SUCCESS] All 4 repository invariants verified.\033[0m")
    sys.exit(0)
else:
    print(f"\033[31m[FAILURE] {violations} invariant violation(s) detected.\033[0m")
    sys.exit(1)
EOF
