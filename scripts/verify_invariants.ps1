<#
.SYNOPSIS
    Automated repository invariants verification for CodeGuards / SlugRails.

.DESCRIPTION
    Validates the four foundational architectural and quality invariants:
    1. Every file in `src/` has <= 200 LOC (hard ceiling).
    2. Zero occurrences of `unsafe` code in `src/` (enforced by `#![forbid(unsafe_code)]`).
    3. Zero test pollution in `src/` (no `#[cfg(test)]` or `#[test]` in `src/`).
    4. All modules declared in `src/guards/checks/mod.rs` exist on disk.

.PARAMETER RepoRoot
    Path to the repository root directory. Defaults to the repository root containing this script.
#>

[CmdletBinding()]
param(
    [string]$RepoRoot = (Resolve-Path "$PSScriptRoot/..").Path
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$violations = 0
$totalChecks = 4

Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host "  SlugRails Repository Invariant Verification" -ForegroundColor Cyan
Write-Host "  Repository Root: $RepoRoot" -ForegroundColor DarkGray
Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host ""

$srcDir = Join-Path $RepoRoot "src"
if (-not (Test-Path $srcDir)) {
    Write-Host "[ERROR] Directory 'src' does not exist at $srcDir" -ForegroundColor Red
    exit 1
}

# Helper to strip comments and string literals (Rust tokenizer equivalent)
function Strip-RustCommentsAndStrings {
    param([string]$Source)

    $chars = $Source.ToCharArray()
    $len = $chars.Length
    $sb = New-Object System.Text.StringBuilder
    $i = 0
    $state = "Normal"
    $rawHashes = 0
    $escaped = $false

    while ($i -lt $len) {
        $c = $chars[$i]
        $next = if ($i + 1 -lt $len) { $chars[$i + 1] } else { [char]0 }

        switch ($state) {
            "Normal" {
                if ($c -eq '/' -and $next -eq '/') {
                    $state = "LineComment"
                    $i += 2
                    continue
                } elseif ($c -eq '/' -and $next -eq '*') {
                    $state = "BlockComment"
                    $i += 2
                    continue
                } elseif ($c -eq 'r' -and ($next -eq '#' -or $next -eq '"')) {
                    $hashes = 0
                    $idx = $i + 1
                    while ($idx -lt $len -and $chars[$idx] -eq '#') {
                        $hashes++
                        $idx++
                    }
                    if ($idx -lt $len -and $chars[$idx] -eq '"') {
                        $state = "RawString"
                        $rawHashes = $hashes
                        $i = $idx + 1
                        continue
                    } else {
                        [void]$sb.Append($c)
                    }
                } elseif ($c -eq '"') {
                    $state = "String"
                    $escaped = $false
                } elseif ($c -eq "'") {
                    $state = "Char"
                    $escaped = $false
                } else {
                    [void]$sb.Append($c)
                }
            }
            "LineComment" {
                if ($c -eq "`n") {
                    $state = "Normal"
                    [void]$sb.Append($c)
                }
            }
            "BlockComment" {
                if ($c -eq '*' -and $next -eq '/') {
                    $state = "Normal"
                    $i += 2
                    continue
                }
            }
            "String" {
                if ($escaped) {
                    $escaped = $false
                } elseif ($c -eq '\') {
                    $escaped = $true
                } elseif ($c -eq '"') {
                    $state = "Normal"
                }
            }
            "Char" {
                if ($escaped) {
                    $escaped = $false
                } elseif ($c -eq '\') {
                    $escaped = $true
                } elseif ($c -eq "'") {
                    $state = "Normal"
                }
            }
            "RawString" {
                if ($c -eq '"') {
                    $matchClose = $true
                    for ($h = 0; $h -lt $rawHashes; $h++) {
                        if ($i + 1 + $h -ge $len -or $chars[$i + 1 + $h] -ne '#') {
                            $matchClose = $false
                            break
                        }
                    }
                    if ($matchClose) {
                        $state = "Normal"
                        $i += 1 + $rawHashes
                        continue
                    }
                }
            }
        }
        $i++
    }
    return $sb.ToString()
}

# -----------------------------------------------------------------------------
# Invariant 1: File Line Budget (<= 200 LOC per file in src/)
# -----------------------------------------------------------------------------
Write-Host "[1/4] Checking File Line Budget (<= 200 LOC)..." -NoNewline
$srcFiles = Get-ChildItem -Path $srcDir -Recurse -Filter "*.rs"
$locViolations = @()
$maxLines = 0
$maxFile = ""

foreach ($f in $srcFiles) {
    $lineCount = (Get-Content -Path $f.FullName).Count
    $relPath = [System.IO.Path]::GetRelativePath($RepoRoot, $f.FullName)
    if ($lineCount -gt $maxLines) {
        $maxLines = $lineCount
        $maxFile = $relPath
    }
    if ($lineCount -gt 200) {
        $locViolations += [PSCustomObject]@{
            File  = $relPath
            Lines = $lineCount
        }
    }
}

if ($locViolations.Count -eq 0) {
    Write-Host " PASS" -ForegroundColor Green
    Write-Host "      Scanned $($srcFiles.Count) files. Max lines: $maxLines ($maxFile)" -ForegroundColor DarkGray
} else {
    Write-Host " FAIL" -ForegroundColor Red
    foreach ($v in $locViolations) {
        Write-Host "      [VIOLATION] $($v.File) has $($v.Lines) lines (> 200 LOC ceiling)" -ForegroundColor Red
    }
    $violations += $locViolations.Count
}

# -----------------------------------------------------------------------------
# Invariant 2: Zero occurrences of `unsafe` code in `src/` (enforced by #![forbid(unsafe_code)])
# -----------------------------------------------------------------------------
Write-Host "[2/4] Checking Zero Unsafe Code & #![forbid(unsafe_code)]..." -NoNewline
$unsafeViolations = @()

# Verify crate root directives
$libRs = Join-Path $srcDir "lib.rs"
$mainRs = Join-Path $srcDir "main.rs"

if (Test-Path $libRs) {
    $libContent = Get-Content -Path $libRs -Raw
    if ($libContent -notmatch '#!\[forbid\(unsafe_code\)\]') {
        $unsafeViolations += "src/lib.rs is missing crate-level '#![forbid(unsafe_code)]'"
    }
} else {
    $unsafeViolations += "src/lib.rs does not exist"
}

if (Test-Path $mainRs) {
    $mainContent = Get-Content -Path $mainRs -Raw
    if ($mainContent -notmatch '#!\[forbid\(unsafe_code\)\]') {
        $unsafeViolations += "src/main.rs is missing crate-level '#![forbid(unsafe_code)]'"
    }
} else {
    $unsafeViolations += "src/main.rs does not exist"
}

# Scan stripped code in all src files for unsafe keywords
foreach ($f in $srcFiles) {
    $rawContent = [System.IO.File]::ReadAllText($f.FullName)
    $stripped = Strip-RustCommentsAndStrings -Source $rawContent
    if ($stripped -match '\bunsafe\b') {
        $relPath = [System.IO.Path]::GetRelativePath($RepoRoot, $f.FullName)
        $unsafeViolations += "Unsafe code token detected in $relPath"
    }
}

if ($unsafeViolations.Count -eq 0) {
    Write-Host " PASS" -ForegroundColor Green
    Write-Host "      #![forbid(unsafe_code)] present in lib.rs & main.rs; zero unsafe blocks detected." -ForegroundColor DarkGray
} else {
    Write-Host " FAIL" -ForegroundColor Red
    foreach ($uv in $unsafeViolations) {
        Write-Host "      [VIOLATION] $uv" -ForegroundColor Red
    }
    $violations += $unsafeViolations.Count
}

# -----------------------------------------------------------------------------
# Invariant 3: Zero test pollution in `src/` (no #[cfg(test)] or #[test] in src/)
# -----------------------------------------------------------------------------
Write-Host "[3/4] Checking Zero Test Pollution in src/..." -NoNewline
$testPollutionViolations = @()

foreach ($f in $srcFiles) {
    $rawContent = [System.IO.File]::ReadAllText($f.FullName)
    $stripped = Strip-RustCommentsAndStrings -Source $rawContent
    if ($stripped -match '#\[\s*(?:cfg\s*\(\s*test\s*\)|test)\s*\]') {
        $relPath = [System.IO.Path]::GetRelativePath($RepoRoot, $f.FullName)
        $testPollutionViolations += "Test attribute (#[test] or #[cfg(test)]) detected in production source: $relPath"
    }
}

if ($testPollutionViolations.Count -eq 0) {
    Write-Host " PASS" -ForegroundColor Green
    Write-Host "      Zero #[cfg(test)] or #[test] attributes in src/ (isolated to tests/)." -ForegroundColor DarkGray
} else {
    Write-Host " FAIL" -ForegroundColor Red
    foreach ($tp in $testPollutionViolations) {
        Write-Host "      [VIOLATION] $tp" -ForegroundColor Red
    }
    $violations += $testPollutionViolations.Count
}

# -----------------------------------------------------------------------------
# Invariant 4: All modules declared in `src/guards/checks/mod.rs` exist on disk
# -----------------------------------------------------------------------------
Write-Host "[4/4] Checking Module Disk Alignment for checks/mod.rs..." -NoNewline
$checksModPath = Join-Path $srcDir "guards/checks/mod.rs"
$moduleViolations = @()

if (-not (Test-Path $checksModPath)) {
    $moduleViolations += "File src/guards/checks/mod.rs does not exist"
} else {
    $checksContent = Get-Content -Path $checksModPath -Raw
    $modMatches = [regex]::Matches($checksContent, '(?m)^\s*(?:pub\s+)?mod\s+([a-zA-Z0-9_]+)\s*;')
    $declaredModules = @($modMatches | ForEach-Object { $_.Groups[1].Value })

    foreach ($modName in $declaredModules) {
        $singleFile = Join-Path $srcDir "guards/checks/$modName.rs"
        $dirModFile = Join-Path $srcDir "guards/checks/$modName/mod.rs"
        if (-not (Test-Path $singleFile) -and -not (Test-Path $dirModFile)) {
            $moduleViolations += "Declared module '$modName' has neither '$singleFile' nor '$dirModFile'"
        }
    }
}

if ($moduleViolations.Count -eq 0) {
    Write-Host " PASS" -ForegroundColor Green
    Write-Host "      All $($declaredModules.Count) declared modules in src/guards/checks/mod.rs exist on disk." -ForegroundColor DarkGray
} else {
    Write-Host " FAIL" -ForegroundColor Red
    foreach ($mv in $moduleViolations) {
        Write-Host "      [VIOLATION] $mv" -ForegroundColor Red
    }
    $violations += $moduleViolations.Count
}

# -----------------------------------------------------------------------------
# Final Summary
# -----------------------------------------------------------------------------
Write-Host ""
Write-Host "-----------------------------------------------------------------" -ForegroundColor DarkGray
if ($violations -eq 0) {
    Write-Host " SUCCESS: All $totalChecks repository invariants passed with 0 errors!" -ForegroundColor Green
    exit 0
} else {
    Write-Host " FAILURE: $violations invariant violation(s) detected across repository!" -ForegroundColor Red
    exit 1
}
